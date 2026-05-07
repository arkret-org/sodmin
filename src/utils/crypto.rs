//! Browser-side crypto helpers: cryptographically secure random bytes
//! via `window.crypto.getRandomValues`, plus a tiny base64url encoder
//! for emitting tokens (PKCE verifier, idempotency keys, OAuth state).

pub fn random_bytes(len: usize) -> Vec<u8> {
    let crypto = web_sys::window()
        .expect("no window")
        .crypto()
        .expect("no crypto");
    let mut buf = vec![0u8; len];
    crypto
        .get_random_values_with_u8_array(&mut buf)
        .expect("get_random_values failed");
    buf
}

pub fn base64url_encode(data: &[u8]) -> String {
    let binary: String = data.iter().map(|&b| b as char).collect();
    let b64 = web_sys::window()
        .expect("no window")
        .btoa(&binary)
        .expect("btoa failed");
    b64.replace('+', "-")
        .replace('/', "_")
        .trim_end_matches('=')
        .to_string()
}

/// Cryptographically random base64url token of `bytes` random octets.
pub fn random_token(bytes: usize) -> String {
    base64url_encode(&random_bytes(bytes))
}

/// Decode a base64url-encoded string (no padding).
pub fn base64url_decode(encoded: &str) -> Option<Vec<u8>> {
    let padded = encoded
        .replace('-', "+")
        .replace('_', "/");
    let padding = (4 - padded.len() % 4) % 4;
    let padded = format!("{}{}", padded, "=".repeat(padding));
    let binary = web_sys::window()?.atob(&padded).ok()?;
    Some(binary.bytes().collect())
}
