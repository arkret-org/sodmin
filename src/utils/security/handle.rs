//! Handle display helpers for admin read-only views.

/// Format the operator-facing display sigil for a canonical handle.
///
/// Given canonical bytes `<prepared-localpart>:<lowercase-A-label-domain>`, returns
/// `@<prepared-localpart>:<lowercase-A-label-domain>`. Already-sigil input is accepted so
/// callers can render mixed data defensively.
pub fn display_sigil(canonical: &str) -> String {
    let body = canonical
        .trim()
        .strip_prefix('@')
        .unwrap_or(canonical.trim());
    format!("@{body}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_sigil_formats() {
        assert_eq!(display_sigil("alice:example.com"), "@alice:example.com");
        assert_eq!(display_sigil("@alice:example.com"), "@alice:example.com");
    }
}
