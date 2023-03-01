use std::{
    ffi::c_int,
    fmt::{Debug, Display},
    fs::OpenOptions,
    io::{BufReader, BufWriter},
    str::FromStr,
};

use ecdsa::elliptic_curve::ScalarCore;
use rand::{rngs::StdRng, SeedableRng};

use anyhow::Result;
use k256::{PublicKey, Secp256k1, SecretKey};
use secp256k1::{
    constants::{PUBLIC_KEY_SIZE, SECRET_KEY_SIZE},
    ffi::{
        types::{c_uchar, c_uint, size_t},
        CPtr, Context,
    },
    Error,
};
use serde::{Deserialize, Serialize};
use web3::{
    signing::keccak256,
    transports::WebSocket,
    types::{Address, TransactionParameters, H256, U256},
    Web3,
};

use crate::utils::{gen_systime, to_eth, to_wei};

pub const SECP256K1_SER_COMPRESSED: c_uint = (1 << 1) | (1 << 8);
pub const UNCOMPRESSED_PUBLIC_KEY_SIZE: usize = 65;

#[derive(Serialize, Deserialize, Debug)]
pub struct Wallet {
    pub secret_key: String,
    pub public_key: String,
    pub public_addr: String,
}

#[derive(Debug)]
struct Idk {
    pub pubk: PublicKey,
}

impl CPtr for Idk {
    type Target = PublicKey;

    fn as_c_ptr(&self) -> *const Self::Target {
        &self.pubk
    }

    fn as_mut_c_ptr(&mut self) -> *mut Self::Target {
        &mut self.pubk
    }
}

extern "C" {
    pub static secp256k1_context_no_precomp: *const Context;
}

extern "C" {
    pub fn secp256k1_ec_pubkey_serialize(
        cx: *const Context,
        output: *mut c_uchar,
        out_len: *mut size_t,
        pk: *const PublicKey,
        compressed: c_uint,
    ) -> c_int;
}

pub fn gen_keypair(rng: ecdsa::elliptic_curve::ScalarCore<Secp256k1>) -> (SecretKey, PublicKey) {
    let seck = SecretKey::new(rng);
    let pubk = PublicKey::from_secret_scalar(&seck.to_nonzero_scalar());
    (seck, pubk)
}

pub fn serialize_internal(pub_key: &PublicKey, ret: &mut [u8], flag: c_uint) {
    let mut ret_len = ret.len();
    let pubk = Idk {
        pubk: pub_key.to_owned(),
    };
    let res = unsafe {
        secp256k1_ec_pubkey_serialize(
            secp256k1_context_no_precomp,
            ret.as_mut_c_ptr(),
            &mut ret_len,
            pubk.as_c_ptr(),
            flag,
        )
    };

    debug_assert_eq!(res, 1);
    debug_assert_eq!(ret_len, ret.len());
}

pub fn serialize_uncompressed(pub_key: &PublicKey) -> [u8; UNCOMPRESSED_PUBLIC_KEY_SIZE] {
    let mut ret = [0u8; UNCOMPRESSED_PUBLIC_KEY_SIZE];
    serialize_internal(pub_key, &mut ret, SECP256K1_SER_COMPRESSED);
    ret
}

pub fn gen_key() -> (SecretKey, PublicKey) {
    let secp = Secp256k1::default();
    let mut rng = StdRng::seed_from_u64(gen_systime().unwrap());
    gen_keypair(ScalarCore::random(rng))
}

pub fn pub_key_addr(pub_key: &PublicKey) -> Address {
    let pub_key = serialize_uncompressed(pub_key);

    debug_assert_eq!(pub_key[0], 0x04);
    let hash = keccak256(&pub_key[1..]);

    Address::from_slice(&hash[12..])
}

pub struct DisplaySecret {
    secret: [u8; SECRET_KEY_SIZE],
}

pub fn to_hex<'a>(src: &[u8], target: &'a mut [u8]) -> Result<String, ()> {
    let hex_len = src.len() * 2;
    if target.len() < hex_len {
        return Err(());
    }
    const HEX_TABLE: [u8; 16] = *b"0123456789abcdef";

    let mut i = 0;
    for &b in src {
        target[i] = HEX_TABLE[usize::from(b >> 4)];
        target[i + 1] = HEX_TABLE[usize::from(b & 0b00001111)];
        i += 2;
    }

    let resl = &target[..hex_len];
    debug_assert!(String::from_utf8(resl.to_vec()).is_ok());
    return unsafe { Ok(String::from_utf8_unchecked(resl.to_vec())) };
}

impl Debug for DisplaySecret {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut slice = [0u8; SECRET_KEY_SIZE * 2];
        let hex = to_hex(&self.secret, &mut slice).expect("We're Fucked!");
        f.debug_tuple("DisplaySecret").field(&hex).finish()
    }
}

impl Display for DisplaySecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.secret {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

pub fn display_sec(sec: &SecretKey) -> DisplaySecret {
    DisplaySecret {
        secret: sec.to_be_bytes().into(),
    }
}

pub fn from_hex(hex: &str, target: &mut [u8]) -> Result<usize, ()> {
    if hex.len() % 2 == 1 || hex.len() > target.len() * 2 {
        return Err(());
    }

    let mut b = 0;
    let mut idx = 0;
    for c in hex.bytes() {
        b <<= 4;
        match c {
            b'A'..=b'F' => b |= c - b'A' + 10,
            b'a'..=b'f' => b |= c - b'a' + 10,
            b'0'..=b'9' => b |= c - b'0',
            _ => return Err(()),
        }

        if (idx & 1) == 1 {
            target[idx / 2] = b;
            b = 0;
        }
        idx += 1;
    }
    Ok(idx / 2)
}

pub fn sec_from_str(s: &str) -> std::result::Result<SecretKey, Error> {
    let mut resl = [0u8; SECRET_KEY_SIZE];
    match from_hex(s, &mut resl) {
        Ok(SECRET_KEY_SIZE) => Ok(SecretKey::from_be_bytes(&resl).expect("We're Fucked!")),
        _ => Err(Error::InvalidSecretKey),
    }
}

pub fn pub_from_str(s: &str) -> Result<PublicKey, Error> {
    let mut resl = [0u8; UNCOMPRESSED_PUBLIC_KEY_SIZE];
    match from_hex(s, &mut resl) {
        Ok(PUBLIC_KEY_SIZE) => {
            Ok(PublicKey::from_sec1_bytes(&resl[0..PUBLIC_KEY_SIZE]).expect("We're Fucked!"))
        }
        Ok(UNCOMPRESSED_PUBLIC_KEY_SIZE) => {
            Ok(PublicKey::from_sec1_bytes(&resl).expect("We're Fucked!"))
        }
        _ => Err(Error::InvalidPublicKey),
    }
}

impl Wallet {
    pub fn new(sec_key: &SecretKey, pub_key: &PublicKey) -> Self {
        let addr = pub_key_addr(&pub_key);
        Wallet {
            secret_key: display_sec(sec_key).to_string(),
            public_key: format!("{:?}", pub_key),
            public_addr: format!("{:?}", addr),
        }
    }

    pub fn save_as_file(&self, path: &str) -> Result<()> {
        let file = OpenOptions::new().write(true).create(true).open(path)?;
        let writer = BufWriter::new(file);

        serde_json::to_writer_pretty(writer, self)?;
        Ok(())
    }

    pub fn load_file(path: &str) -> Result<Wallet> {
        let file = OpenOptions::new().read(true).open(path)?;
        let reader = BufReader::new(file);

        let wallet = serde_json::from_reader(reader)?;
        Ok(wallet)
    }

    pub fn sec_key(&self) -> Result<SecretKey> {
        let sec_key = sec_from_str(&self.secret_key)?;
        Ok(sec_key)
    }

    pub fn pub_key(&self) -> Result<PublicKey> {
        let pub_key = pub_from_str(&self.public_key)?;
        Ok(pub_key)
    }

    pub async fn get_balance(&self, web3_conc: &Web3<WebSocket>) -> Result<U256> {
        let wallet_addr = Address::from_str(&self.public_addr)?;
        let balance = web3_conc.eth().balance(wallet_addr, None).await?;
        Ok(balance)
    }

    pub async fn get_balance_as_eth(&self, web3_conc: &Web3<WebSocket>) -> Result<f64> {
        let val = self.get_balance(web3_conc).await?;
        Ok(to_eth(val))
    }

    pub async fn sign_send(
        web3_conc: &Web3<WebSocket>,
        transc: TransactionParameters,
        sec_key: &SecretKey,
    ) -> Result<H256> {
        let signed = web3_conc
            .accounts()
            .sign_transaction(transc, sec_key)
            .await?;

        let tranc_resl = web3_conc
            .eth()
            .send_raw_transaction(signed.raw_transaction)
            .await?;
        Ok(tranc_resl)
    }
}

pub async fn connect_web3(url: &str) -> Result<Web3<WebSocket>> {
    let transport = WebSocket::new(url).await?;
    Ok(Web3::new(transport))
}

pub fn send_eth(addr: Address, valu: f64) -> TransactionParameters {
    TransactionParameters {
        to: Some(addr),
        value: to_wei(valu),
        ..Default::default()
    }
}
