mod eth_wallet;
mod utils;
use crate::eth_wallet::{connect_web3, Wallet};
use anyhow::Result;
use dotenvy::{dotenv, var};

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

    Ok(())
}
