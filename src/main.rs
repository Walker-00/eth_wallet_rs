mod eth_wallet;
use anyhow::Result;
use eth_wallet::gen_key;

use crate::eth_wallet::{pub_key_addr, Wallet};

#[tokio::main]
async fn main() -> Result<()> {
    let (sec_key, pub_key) = gen_key();

    println!(
        "sec_key:    {}\npub_key:    {}",
        sec_key.display_secret(),
        pub_key.to_string()
    );

    let pub_addr = pub_key_addr(&pub_key);

    print!("addr:    {:?}", pub_addr);

    let wallet = Wallet::new(&sec_key, &pub_key);
    println!("{:?}", wallet);
    let path = "wallet.json";
    wallet.save_as_file(path)?;

    Wallet::load_file(path)?;

    Ok(())
}
