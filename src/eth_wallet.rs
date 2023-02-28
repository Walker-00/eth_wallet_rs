use std::{
    fs::OpenOptions,
    io::{BufReader, BufWriter},
};

use anyhow::Result;
use secp256k1::{
    rand::{rngs::StdRng, SeedableRng},
    PublicKey, Secp256k1, SecretKey,
};
use serde::{Deserialize, Serialize};
use web3::{signing::keccak256, types::Address};

#[derive(Serialize, Deserialize, Debug)]
pub struct Wallet {
    pub secret_key: String,
    pub public_key: String,
    pub public_addr: String,
}

pub fn gen_key() -> (SecretKey, PublicKey) {
    let secp = Secp256k1::new();
    let mut rng = StdRng::seed_from_u64(111);
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
}
