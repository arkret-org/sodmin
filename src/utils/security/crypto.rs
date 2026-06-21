//! Browser-side crypto helpers: cryptographically secure random bytes
//! via `window.crypto.getRandomValues`, plus base64url encode/decode for
//! emitting and parsing tokens (PKCE verifier, idempotency keys, OAuth
//! state, id_token payloads).
//!
//! base64url is handled by the pure-Rust `base64` crate (`URL_SAFE_NO_PAD`
//! engine) rather than the host `btoa`/`atob`: that avoids the Latin-1
//! round-trip corruption `atob` causes for bytes > 0x7F (which silently
//! mangled JWT payloads) and removes a JS boundary call from the hot path.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// Generate `len` cryptographically-secure random octets.
///
/// `window.crypto.getRandomValues` is a hard requirement for a security
/// primitive — there is no safe fallback. In any real browser context
/// these are always available; a failure means the environment cannot
/// produce secure randomness, so we fail closed (panic) rather than
/// silently emit predictable bytes.
pub fn random_bytes(len: usize) -> Vec<u8> {
    let crypto = web_sys::window()
        .expect("no window")
        .crypto()
        .expect("no window.crypto — insecure environment");
    let mut buf = vec![0u8; len];
    crypto
        .get_random_values_with_u8_array(&mut buf)
        .expect("crypto.getRandomValues failed");
    buf
}

pub fn base64url_encode(data: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(data)
}

/// Cryptographically random base64url token of `bytes` random octets.
pub fn random_token(bytes: usize) -> String {
    base64url_encode(&random_bytes(bytes))
}

/// Decode a base64url-encoded string (no padding). Tolerates a stray
/// trailing `=` pad by stripping it first.
pub fn base64url_decode(encoded: &str) -> Option<Vec<u8>> {
    let trimmed = encoded.trim_end_matches('=');
    URL_SAFE_NO_PAD.decode(trimmed.as_bytes()).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64url_round_trip_all_byte_values() {
        // The old btoa/atob path corrupted bytes > 0x7F; this covers the
        // full 0x00–0xFF range to lock in the fix.
        let data: Vec<u8> = (0u16..=255).map(|b| b as u8).collect();
        let encoded = base64url_encode(&data);
        assert!(!encoded.contains('+') && !encoded.contains('/') && !encoded.contains('='));
        assert_eq!(base64url_decode(&encoded), Some(data));
    }

    #[test]
    fn base64url_decode_tolerates_padding() {
        // "Zm9v" == "foo"; with explicit padding stripped.
        assert_eq!(base64url_decode("Zm9v"), Some(b"foo".to_vec()));
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use wasm_bindgen_test::*;

    use super::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn random_bytes_uses_browser_crypto() {
        let first = random_bytes(32);
        let second = random_bytes(32);
        assert_eq!(first.len(), 32);
        assert_eq!(second.len(), 32);
        assert!(first.iter().any(|byte| *byte != 0));
        assert_ne!(first, second);
    }
}
