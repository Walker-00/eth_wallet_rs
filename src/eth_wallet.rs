use secp256k1::{
    rand::{rngs::StdRng, SeedableRng},
    PublicKey, Secp256k1, SecretKey,
};
use web3::{signing::keccak256, types::Address};

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
