pub const DESTRUCTIVE_REASON_MAX_CHARS: usize = 512;

/// Validate a short, single-line audit reason without accepting obvious
/// credential material. Backends enforce the same policy before persistence.
pub fn destructive_reason_error(reason: &str, required: bool) -> Option<&'static str> {
    let reason = reason.trim();
    if reason.is_empty() {
        return required.then_some("dangerous_action.reason_required");
    }
    if reason.chars().count() > DESTRUCTIVE_REASON_MAX_CHARS {
        return Some("dangerous_action.reason_too_long");
    }
    if reason.chars().any(char::is_control) {
        return Some("dangerous_action.reason_single_line");
    }

    let lower = reason.to_ascii_lowercase();
    const SENSITIVE_MARKERS: &[&str] = &[
        "-----begin private key",
        "authorization:",
        "bearer ey",
        "password=",
        "secret=",
        "token=",
    ];
    if SENSITIVE_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
    {
        return Some("dangerous_action.reason_sensitive");
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_reason_rejects_empty_and_sensitive_values() {
        assert_eq!(
            destructive_reason_error("  ", true),
            Some("dangerous_action.reason_required")
        );
        assert_eq!(
            destructive_reason_error("Authorization: Bearer eyJhbGciOi", true),
            Some("dangerous_action.reason_sensitive")
        );
        assert_eq!(
            destructive_reason_error("SEC-1234 account recovery", true),
            None
        );
    }

    #[test]
    fn optional_reason_accepts_empty_but_rejects_multiline() {
        assert_eq!(destructive_reason_error("", false), None);
        assert_eq!(
            destructive_reason_error("first line\nsecond line", false),
            Some("dangerous_action.reason_single_line")
        );
    }
}
