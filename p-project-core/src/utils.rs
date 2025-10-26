use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

/// Generate a random string ID
pub fn generate_id() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

/// Convert wallet address to a shorter display format
pub fn shorten_wallet_address(address: &str) -> String {
    if address.len() <= 10 {
        return address.to_string();
    }
    format!("{}...{}", &address[..6], &address[address.len()-4..])
}