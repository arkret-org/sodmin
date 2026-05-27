//! Round 4 — DID input validation against the tightened method-name
//! grammar (`^did:[a-z0-9]+:[^\s]+$`).
//!
//! Spec a77b995 narrowed the DID method segment from
//! `[a-z0-9:.\-_]+` to `[a-z0-9]+`: no `.`, `-`, `_`, or `:` inside
//! the method name. The body (after the second `:`) is still any run
//! of non-whitespace characters. Every DID input on the sodmin surface
//! MUST validate against this regex before submission so the admin
//! gets the same error the SDK would emit on the wire.
//!
//! `regex_lite` is used (already a sodmin dependency) so we do not pull
//! the full `regex` crate into the wasm bundle.

use regex_lite::Regex;
use std::sync::OnceLock;

/// Round 4 — tightened DID grammar. `method` segment is now `[a-z0-9]+`
/// only; the body after the second `:` is any non-whitespace.
pub const DID_REGEX_PATTERN: &str = r"^did:[a-z0-9]+:[^\s]+$";

fn did_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(DID_REGEX_PATTERN).expect("DID_REGEX_PATTERN is valid"))
}

/// Returns `true` when `s` is a DID under the round-4 tightened grammar.
///
/// The empty string is rejected. Any whitespace (including a trailing
/// newline) is rejected. The method segment MUST be lowercase ASCII
/// alphanumeric — no `.`, `-`, `_`, or `:`.
pub fn is_valid_did(s: &str) -> bool {
    !s.is_empty() && did_regex().is_match(s)
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
        assert!(!is_valid_did("did:web.vh:alice")); // ROUND4-ALLOW: negative test
        assert!(!is_valid_did("did:web-vh:alice")); // ROUND4-ALLOW: negative test
        assert!(!is_valid_did("did:web_vh:alice")); // ROUND4-ALLOW: negative test
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
