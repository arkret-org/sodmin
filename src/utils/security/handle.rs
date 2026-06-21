//! R3 — Handle homograph / NFC localpart validator.
//!
//! Mirrors the SDK helper `cokret_core::models::handle::
//! normalize_handle_localpart` so the admin UI can flag a suspect
//! handle inline before submission instead of waiting for a server-side
//! `handle_homograph_forbidden` round-trip.
//!
//! The check delegates to the SDK helper instead of carrying a
//! sodmin-local homograph table.
//!
//! `is_safe_handle_localpart` returns `Ok(())` when the input is clean,
//! `Err(reason)` otherwise. The full UTS#39 skeleton table is the SDK's
//! responsibility (R3.1); this helper is just enough to render an inline
//! "looks suspicious" warning under the input field.
//!
//! ## R3.1 — canonical handle wire form
//!
//! The canonical wire form is `<localpart>:<domain>(:<port>)?` — the
//! pre-R3.1 `cokret://<domain>/users/<localpart>` URI form has been
//! retired (cokret-spec @ 7157ee8). The admin UI MAY render the
//! display sigil `@<localpart>:<domain>` to operators, but MUST
//! normalize back to the canonical bytes via
//! [`normalize_to_canonical`] before submitting to soland so soland's
//! `ck.handle.*` reducers see the wire shape they verify against.
//!
//! See `cokret_core::models::handle::Handle` for the SDK-side parser /
//! formatter; this module is the admin-SPA mirror.

/// Returns `Ok(())` when `input` looks like a clean ASCII / pure-script
/// handle localpart. On rejection returns a short reason tag that
/// pages can map through i18n for an inline warning.
pub fn is_safe_handle_localpart(input: &str) -> Result<(), HomographReason> {
    cokret_core::models::normalize_handle_localpart(input)
        .map(|_| ())
        .map_err(|err| classify_handle_normalize_error(&err.to_string()))
}

fn classify_handle_normalize_error(message: &str) -> HomographReason {
    let lower = message.to_ascii_lowercase();
    if lower.contains("length") || lower.contains("empty") {
        HomographReason::OutOfRange
    } else if lower.contains("zero-width") || lower.contains("bidi") {
        HomographReason::ZeroWidthOrBidi
    } else {
        HomographReason::Confusable
    }
}

/// Tag returned for each rejection reason — surfaced to the UI as a
/// short message above the input field. The full diagnostic text lives
/// in `utils::i18n` so the strings can be localized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomographReason {
    OutOfRange,
    ZeroWidthOrBidi,
    Confusable,
}

impl HomographReason {
    pub fn i18n_key(self) -> &'static str {
        match self {
            HomographReason::OutOfRange => "error.handle_homograph_out_of_range",
            HomographReason::ZeroWidthOrBidi => "error.handle_homograph_zero_width",
            HomographReason::Confusable => "error.handle_homograph_confusable",
        }
    }
}

/// R3.1 — normalize any of the operator-visible handle spellings down
/// to the canonical wire form `<localpart>:<domain>(:<port>)?` that
/// soland verifies against.
///
/// Accepts:
/// - canonical `localpart:domain[:port]` (returned as-is, lower-cased)
/// - display sigil `@localpart:domain[:port]` (strip leading `@`)
/// - interop `acct:localpart@domain[:port]` (rewrite to canonical)
/// - retired `cokret://domain/users/localpart` URI form (rewrite to canonical; the admin UI is the
///   last surface where this can leak in from a copy-paste, so we accept it as input but never emit
///   it)
///
/// Returns `Err` when the input is empty after trimming or carries
/// structural noise we can't recover from (e.g. multiple `@`). The
/// caller is expected to additionally run [`is_safe_handle_localpart`]
/// against the resulting localpart to catch homograph attacks.
pub fn normalize_to_canonical(input: &str) -> Result<String, HandleNormalizeError> {
    use cokret_core::models::Handle;

    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(HandleNormalizeError::Empty);
    }

    // Global report #10 (candidate 8) — the spelling-recovery (prefix
    // stripping / `acct:` rewrite) stays here because it is an
    // admin-UI-only convenience the SDK parser does not perform, but the
    // authoritative localpart / domain / port validation + canonical
    // lowercasing is delegated to the SDK `Handle` parser so the admin UI
    // accepts exactly what soland's `ck.handle.*` reducers verify.

    // 1. Retired `cokret://` URI form. We accept on input so a stale bookmark / copy-paste
    //    round-trips into canonical; we never emit it on output.
    if let Some(rest) = trimmed.strip_prefix("cokret://") {
        let mut parts = rest.splitn(3, '/');
        let domain = parts.next().unwrap_or("");
        let users = parts.next().unwrap_or("");
        let localpart = parts.next().unwrap_or("");
        if users != "users" || localpart.is_empty() || domain.is_empty() {
            return Err(HandleNormalizeError::Malformed);
        }
        return Handle::parse(&format!("{localpart}:{domain}"))
            .map(|h| h.canonical().to_owned())
            .map_err(|_| HandleNormalizeError::Malformed);
    }

    // 2. `acct:` interop form — delegated wholesale to the SDK `Handle::from_acct`, which rewrites
    //    `@` to `:` and validates.
    if trimmed.starts_with("acct:") {
        return Handle::from_acct(trimmed)
            .map(|h| h.canonical().to_owned())
            .map_err(|_| HandleNormalizeError::Malformed);
    }

    // 3. Display sigil `@localpart:domain[:port]` — strip the UI sigil.
    let body = trimmed.strip_prefix('@').unwrap_or(trimmed);

    // 4. Canonical form: hand the bytes to the SDK parser for the authoritative
    //    `<localpart>:<domain>(:<port>)?` validation.
    Handle::parse(body)
        .map(|h| h.canonical().to_owned())
        .map_err(|_| HandleNormalizeError::Malformed)
}

/// R3.1 — format the operator-facing display sigil for a canonical
/// handle. Given canonical bytes `<localpart>:<domain>(:<port>)?` it
/// returns `@<localpart>:<domain>(:<port>)?`. Pass any operator input
/// through [`normalize_to_canonical`] first if you're not certain it's
/// already canonical.
pub fn display_sigil(canonical: &str) -> String {
    let body = canonical
        .trim()
        .strip_prefix('@')
        .unwrap_or(canonical.trim());
    format!("@{body}")
}

/// R3.1 — failure reason from [`normalize_to_canonical`]. Surfaces a
/// short i18n key the page can render under the input field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleNormalizeError {
    Empty,
    Malformed,
}

impl HandleNormalizeError {
    pub fn i18n_key(self) -> &'static str {
        match self {
            HandleNormalizeError::Empty => "error.handle_empty",
            HandleNormalizeError::Malformed => "error.handle_malformed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ascii_handles() {
        assert!(is_safe_handle_localpart("alice").is_ok());
        assert!(is_safe_handle_localpart("alice_1").is_ok());
        assert!(is_safe_handle_localpart("alice-bob").is_ok());
    }

    #[test]
    fn rejects_script_mixed() {
        // Latin 'a' followed by Cyrillic small letter "а" (U+0430).
        let suspicious = "alic\u{0430}";
        assert_eq!(
            is_safe_handle_localpart(suspicious),
            Err(HomographReason::Confusable)
        );
    }

    #[test]
    fn rejects_confusable() {
        // Pure-Cyrillic localpart that still looks like "paypal" — even
        // without script mixing it MUST be rejected so the homograph
        // bypass doesn't slip through.
        let pure_cyrillic = "\u{0440}\u{0430}\u{0443}\u{0440}\u{0430}\u{0435}";
        assert_eq!(
            is_safe_handle_localpart(pure_cyrillic),
            Err(HomographReason::Confusable)
        );
    }

    #[test]
    fn rejects_zero_width() {
        let with_zwsp = "alice\u{200B}bob";
        assert_eq!(
            is_safe_handle_localpart(with_zwsp),
            Err(HomographReason::ZeroWidthOrBidi)
        );
    }

    #[test]
    fn rejects_empty_and_oversize() {
        assert_eq!(
            is_safe_handle_localpart(""),
            Err(HomographReason::OutOfRange)
        );
        let too_long = "a".repeat(129);
        assert_eq!(
            is_safe_handle_localpart(&too_long),
            Err(HomographReason::OutOfRange)
        );
    }

    #[test]
    fn normalizes_canonical_passthrough() {
        assert_eq!(
            normalize_to_canonical("alice:example.com").unwrap(),
            "alice:example.com"
        );
        assert_eq!(
            normalize_to_canonical("Alice:Example.COM").unwrap(),
            "alice:example.com"
        );
        assert_eq!(
            normalize_to_canonical("alice:example.com:8443").unwrap(),
            "alice:example.com:8443"
        );
    }

    #[test]
    fn normalizes_display_sigil() {
        assert_eq!(
            normalize_to_canonical("@alice:example.com").unwrap(),
            "alice:example.com"
        );
        assert_eq!(
            normalize_to_canonical("  @alice:example.com  ").unwrap(),
            "alice:example.com"
        );
    }

    #[test]
    fn normalizes_acct_interop() {
        assert_eq!(
            normalize_to_canonical("acct:alice@example.com").unwrap(),
            "alice:example.com"
        );
    }

    #[test]
    fn normalizes_retired_cokret_uri() {
        assert_eq!(
            normalize_to_canonical("cokret://example.com/users/alice").unwrap(),
            "alice:example.com"
        );
    }

    #[test]
    fn rejects_malformed_handles() {
        assert_eq!(normalize_to_canonical(""), Err(HandleNormalizeError::Empty));
        assert_eq!(
            normalize_to_canonical("alice"),
            Err(HandleNormalizeError::Malformed)
        );
        assert_eq!(
            normalize_to_canonical("alice:example.com:443:extra"),
            Err(HandleNormalizeError::Malformed)
        );
        assert_eq!(
            normalize_to_canonical("acct:alice"),
            Err(HandleNormalizeError::Malformed)
        );
    }

    #[test]
    fn display_sigil_formats() {
        assert_eq!(display_sigil("alice:example.com"), "@alice:example.com");
        // Already-sigil input is idempotent so callers can be sloppy.
        assert_eq!(display_sigil("@alice:example.com"), "@alice:example.com");
    }
}
