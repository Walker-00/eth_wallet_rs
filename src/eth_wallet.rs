use std::{
    fs::OpenOptions,
    io::{BufReader, BufWriter},
    str::FromStr,
};

use anyhow::Result;
use secp256k1::{
    rand::{rngs::StdRng, SeedableRng},
    PublicKey, Secp256k1, SecretKey,
};
use serde::{Deserialize, Serialize};
use web3::{
    signing::{keccak256, SecretKey, SecretKeyRef},
    transports::WebSocket,
    types::{Address, TransactionParameters, H256, U256},
    Web3,
};

use crate::utils::{gen_systime, to_eth, to_wei};

#[derive(Serialize, Deserialize, Debug)]
pub struct Wallet {
    pub secret_key: String,
    pub public_key: String,
    pub public_addr: String,
}

pub fn gen_key() -> (SecretKey, PublicKey) {
    let secp = Secp256k1::new();
    let mut rng = StdRng::seed_from_u64(gen_systime().unwrap());
    secp.generate_keypair(&mut rng)
}

pub fn pub_key_addr(pub_key: &PublicKey) -> Address {
    let pub_key = pub_key.serialize_uncompressed();

    debug_assert_eq!(pub_key[0], 0x04);
    let hash = keccak256(&pub_key[1..]);

    Address::from_slice(&hash[12..])
}

impl Wallet {
    pub fn new(sec_key: &SecretKey, pub_key: &PublicKey) -> Self {
        let addr = pub_key_addr(&pub_key);
        Wallet {
            secret_key: sec_key.display_secret().to_string(),
            public_key: pub_key.to_string(),
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
        let sec_key = SecretKey::from_str(&self.secret_key)?;
        Ok(sec_key)
    }

    pub fn pub_key(&self) -> Result<PublicKey> {
        let pub_key = PublicKey::from_str(&self.public_key)?;
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
        sec_key: web3::signing::SecretKey,
    ) -> Result<H256> {
        let signed = web3_conc.accounts().sign_transaction(transc).await?;

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
