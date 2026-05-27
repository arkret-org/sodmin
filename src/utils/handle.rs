//! R3 — Handle homograph / NFC localpart validator.
//!
//! Mirrors the SDK helper `contrix_core::model::handle::
//! normalize_handle_localpart` so the admin UI can flag a suspect
//! handle inline before submission instead of waiting for a server-side
//! `handle_homograph_forbidden` round-trip.
//!
//! The check is intentionally lightweight and conservative:
//!
//! 1. Length must be in `1..=128` bytes.
//! 2. Reject zero-width / bidi control codepoints (U+200B..U+200F,
//!    U+202A..U+202E, U+2060..U+2069, U+FEFF).
//! 3. Reject script-mixed labels (ASCII Latin letters mixed with
//!    non-ASCII letters).
//! 4. Reject the minimal-confusable subset (a small hand-rolled table of
//!    Cyrillic/Greek codepoints that visually fold to ASCII letters).
//!
//! `is_safe_handle_localpart` returns `Ok(())` when the input is clean,
//! `Err(reason)` otherwise. The full UTS#39 skeleton table is the SDK's
//! responsibility (R3.1); this helper is just enough to render an inline
//! "looks suspicious" warning under the input field.

/// Static i18n key returned by [`is_safe_handle_localpart`] when the
/// input would trip the server-side `handle_homograph_forbidden`
/// guard. The page renders the matching string from `utils::i18n`.
#[allow(dead_code)]
pub const HOMOGRAPH_I18N_KEY: &str = "error.handle_homograph_forbidden";

/// Returns `Ok(())` when `input` looks like a clean ASCII / pure-script
/// handle localpart. On rejection returns a short reason tag that
/// pages can map through i18n for an inline warning.
pub fn is_safe_handle_localpart(input: &str) -> Result<(), HomographReason> {
    if input.is_empty() || input.len() > 128 {
        return Err(HomographReason::OutOfRange);
    }

    for ch in input.chars() {
        if matches!(
            ch,
            '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2069}'
            | '\u{FEFF}'
        ) {
            return Err(HomographReason::ZeroWidthOrBidi);
        }
    }

    let mut has_ascii_letter = false;
    let mut has_non_ascii_letter = false;
    for ch in input.chars() {
        if ch.is_ascii_alphabetic() {
            has_ascii_letter = true;
        } else if !ch.is_ascii() && ch.is_alphabetic() {
            has_non_ascii_letter = true;
        }
    }
    if has_ascii_letter && has_non_ascii_letter {
        return Err(HomographReason::ScriptMixed);
    }

    for ch in input.chars() {
        if !ch.is_ascii() && minimal_confusable_for(ch).is_some() {
            return Err(HomographReason::Confusable);
        }
    }

    Ok(())
}

/// Minimal confusable folds — same subset as the SDK R3 helper.
fn minimal_confusable_for(ch: char) -> Option<char> {
    match ch {
        // Cyrillic small letters that fold to ASCII look-alikes.
        '\u{0430}' => Some('a'),
        '\u{0435}' => Some('e'),
        '\u{043E}' => Some('o'),
        '\u{0440}' => Some('p'),
        '\u{0441}' => Some('c'),
        '\u{0443}' => Some('y'),
        '\u{0445}' => Some('x'),
        // Greek small letters with ASCII look-alikes.
        '\u{03B1}' => Some('a'),
        '\u{03BF}' => Some('o'),
        _ => None,
    }
}

/// Tag returned for each rejection reason — surfaced to the UI as a
/// short message above the input field. The full diagnostic text lives
/// in `utils::i18n` so the strings can be localized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomographReason {
    OutOfRange,
    ZeroWidthOrBidi,
    ScriptMixed,
    Confusable,
}

impl HomographReason {
    pub fn i18n_key(self) -> &'static str {
        match self {
            HomographReason::OutOfRange => "error.handle_homograph_out_of_range",
            HomographReason::ZeroWidthOrBidi => "error.handle_homograph_zero_width",
            HomographReason::ScriptMixed => "error.handle_homograph_script_mixed",
            HomographReason::Confusable => "error.handle_homograph_confusable",
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
            Err(HomographReason::ScriptMixed)
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
}
