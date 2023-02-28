use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use web3::types::U256;

pub fn gen_systime() -> Result<u64> {
    let dur = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(dur.as_secs() << 30 | dur.subsec_nanos() as u64)
}

pub fn to_eth(val: U256) -> f64 {
    let resl = val.as_u128() as f64;

    resl / 1_000_000_000_000_000_000.0
}
