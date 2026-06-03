//! Device-id generation. Backed by the same cryptographically-secure
//! entropy source (`crate::utils::security::crypto::random_bytes`) as PKCE / state
//! / nonce so there is a single random source in the codebase rather
//! than a second, non-cryptographic `Math.random()` path.

use crate::utils::security::crypto::random_bytes;

const ALPHANUM: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

fn random_string(charset: &str, length: usize) -> String {
    let chars: Vec<char> = charset.chars().collect();
    random_bytes(length)
        .into_iter()
        .map(|b| chars[(b as usize) % chars.len()])
        .collect()
}

pub fn generate_device_id() -> String {
    random_string(ALPHANUM, 16)
}
