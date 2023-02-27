use secp256k1::{
    rand::{rngs::StdRng, SeedableRng},
    PublicKey, Secp256k1, SecretKey,
};

pub fn gen_key() -> (SecretKey, PublicKey) {
    let secp = Secp256k1::new();
    let mut rng = StdRng::seed_from_u64(1121);
    secp.generate_keypair(&mut rng)
}
