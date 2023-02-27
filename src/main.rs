mod eth_wallet;
use eth_wallet::gen_key;

use crate::eth_wallet::pub_key_addr;

fn main() {
    let (sec_key, pub_key) = gen_key();

    println!(
        "sec_key:    {:?}\npub_key:    {}",
        sec_key,
        pub_key.to_string()
    );

    let pub_addr = pub_key_addr(&pub_key);

    print!("addr:    {:?}", pub_addr);
}
