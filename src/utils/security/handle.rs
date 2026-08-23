//! Handle display helpers for admin read-only views.

/// Format the operator-facing display sigil for a canonical handle.
///
/// Given canonical bytes `<prepared-localpart>:<lowercase-A-label-domain>`, returns
/// `@<prepared-localpart>:<lowercase-A-label-domain>`. The input is already
/// canonical wire data from Soland; this helper does not normalize alternate
/// spellings.
pub fn display_sigil(canonical: &str) -> String {
    format!("@{canonical}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_sigil_formats() {
        assert_eq!(display_sigil("alice:example.com"), "@alice:example.com");
    }
}
