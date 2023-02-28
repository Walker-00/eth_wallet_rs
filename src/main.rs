mod eth_wallet;
mod utils;
use std::str::FromStr;

use crate::eth_wallet::{connect_web3, send_eth, Wallet};
use anyhow::Result;
use dotenvy::{dotenv, var};
use web3::{signing::SecretKeyRef, types::Address};
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().expect("Error due to: .env File not found");

    let path = "wallet.json";

    let wallet_load = Wallet::load_file(path)?;

    let url = var("NET_URL")?;

    let web3_conc = connect_web3(&url).await?;

    let blk_num = web3_conc.eth().block_number().await?;
    println!("block_number: {}", &blk_num);

    let blc = wallet_load.get_balance(&web3_conc).await?;
    println!("wallet balance: {}", &blc);

    let eblc = wallet_load.get_balance_as_eth(&web3_conc).await?;
    println!("ETH balance: {}", &eblc);

    let transc = send_eth(
        Address::from_str("0x4Ba72B5500cd7e0d80f6b5EEEAed7005744fc4F85")?,
        0.001,
    );

    let tranc_hash = Wallet::sign_send(&web3_conc, transc).await?;

    Ok(())
}
