use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use web3::types::U256;

pub fn gen_systime() -> Result<u64> {
    let dur = SystemTime::now().duration_since(UNIX_EPOCH)?;
    Ok(dur.as_secs() << 30 | dur.subsec_nanos() as u64)
}
