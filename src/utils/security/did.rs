//! DID input validation. Global report #10 (candidate 12) — the local
//! `regex_lite`-backed validator has been replaced by the SDK's
//! canonical scalar validator `cokret_identifiers::is_did`.
//!
//! Spec a77b995 narrowed the DID method segment to `[a-z0-9]+`: no `.`,
//! `-`, `_`, or `:` inside the method name. The body (after the second
//! `:`) is any run of non-whitespace characters. `cokret_identifiers::
//! is_did` enforces exactly this grammar (and additionally rejects the
//! `uuid` method and `#`/`?` markers reserved for the DID-URL surface),
//! so every DID input on the sodmin surface validates against the same
//! rule the SDK uses on the wire — no hand-copied regex to drift.

pub use cokret_identifiers::is_did;

/// Returns `true` when `s` is a valid DID scalar.
///
/// Thin wrapper over [`cokret_identifiers::is_did`] kept under the
/// historical sodmin name so existing call sites need no change. The
/// empty string, any whitespace, and uppercase / punctuated method
/// segments are rejected; the method segment MUST be lowercase ASCII
/// alphanumeric.
pub fn is_valid_did(s: &str) -> bool {
    is_did(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_round4_canonical_dids() {
        assert!(is_valid_did("did:web:alice.example"));
        assert!(is_valid_did("did:key:z6MkXYZ"));
        assert!(is_valid_did("did:webvh:auth.example.net:abc123"));
        // The body after the second `:` may contain dots and other
        // non-whitespace.
        assert!(is_valid_did("did:web:foo.bar.example:user-1_v2"));
    }

    #[test]
    fn rejects_pre_round4_method_segments() {
        // Pre-round-4 permitted `.`, `-`, `_`, `:` in the method
        // segment; round 4 wire-breaks this.
        assert!(!is_valid_did("did:web.vh:alice")); // DRIFT-ALLOW: negative test
        assert!(!is_valid_did("did:web-vh:alice")); // DRIFT-ALLOW: negative test
        assert!(!is_valid_did("did:web_vh:alice")); // DRIFT-ALLOW: negative test
        assert!(!is_valid_did("did:Web:alice")); // upper-case rejected.
    }

    #[test]
    fn rejects_obvious_garbage() {
        assert!(!is_valid_did(""));
        assert!(!is_valid_did("alice@example.com"));
        assert!(!is_valid_did("did:"));
        assert!(!is_valid_did("did:web:"));
        assert!(!is_valid_did("did:web:has whitespace"));
        assert!(!is_valid_did("did:web:trailing\nnewline"));
    }
}
