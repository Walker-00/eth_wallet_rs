mod eth_wallet;
use eth_wallet::gen_key;

fn main() {
    let (sec_key, pub_key) = gen_key();

    println!(
        "sec_key:  {}\npub_key:    {}",
        sec_key.display_secret(),
        pub_key.to_string()
    );
}
