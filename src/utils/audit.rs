//! Client-side admin audit helper
//!
//! Why this exists: per-row destructive mutations on the F1/F2/F3
//! Applets / Agents / Directory admin pages must leave a structured
//! breadcrumb in the browser console + any attached log sink, so an
//! operator can correlate a sodmin click with the soland-side audit row
//! it produced. The actual audit-log row is appended *server-side* by
//! soland on receipt of `POST /api/admin/v1/{resource}/{id}/{action}`;
//! this client-side trace is purely a defensive UX aid (correlation +
//! "did the click actually fire" debugging).
//!
//! Output format is a single `log::info!` line shaped like:
//! ```text
//! sodmin.admin.action target=applet id=… action=approve outcome=accepted note=…
//! ```
//!
//! The format is stable so a downstream `dioxus-logger` sink can parse
//! it without re-deriving the schema.

/// Outcome of an admin click — the toast variant on the page mirrors
/// this. `Accepted` means the soland HTTP call returned 2xx; `Rejected`
/// means it returned a non-2xx that wasn't a 404; `NotWired` means the
/// soland surface returned 404 (i.e. soland hasn't shipped this admin
/// endpoint yet, so the click was a no-op).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminAuditOutcome {
    Accepted,
    Rejected,
    NotWired,
}

impl AdminAuditOutcome {
    pub fn label(&self) -> &'static str {
        match self {
            AdminAuditOutcome::Accepted => "accepted",
            AdminAuditOutcome::Rejected => "rejected",
            AdminAuditOutcome::NotWired => "not_wired",
        }
    }

    /// Map an `HttpError.status` to an outcome. `0` and `2xx` count as
    /// `Accepted` (the call site only invokes this on the failure
    /// branch).
    pub fn from_http_status(status: u16) -> Self {
        match status {
            404 => AdminAuditOutcome::NotWired,
            200..=299 => AdminAuditOutcome::Accepted,
            _ => AdminAuditOutcome::Rejected,
        }
    }
}

/// Format an admin audit line. Pure helper so we can unit-test the
/// shape without spinning up a `log::` sink.
///
/// `target` is the resource name (`applet`, `agent`, `directory`),
/// `id` is the row identity from the listing, `action` is the click
/// verb (`approve` / `suspend` / `revoke` / `reject`), `outcome` mirrors
/// the HTTP outcome, and `note` is an optional operator-supplied string.
pub fn format_admin_audit_line(
    target: &str,
    id: &str,
    action: &str,
    outcome: AdminAuditOutcome,
    note: Option<&str>,
) -> String {
    let note_part = match note.map(str::trim).filter(|s| !s.is_empty()) {
        Some(n) => format!(" note={}", redact_note(n)),
        None => String::new(),
    };
    format!(
        "sodmin.admin.action target={} id={} action={} outcome={}{}",
        target,
        id,
        action,
        outcome.label(),
        note_part,
    )
}

/// Truncate / sanitize an operator note for the log line. Strips
/// newlines (so the line stays single-line / grep-friendly) and clamps
/// to 120 chars.
fn redact_note(note: &str) -> String {
    let cleaned: String = note
        .chars()
        .map(|c| match c {
            '\n' | '\r' | '\t' => ' ',
            _ => c,
        })
        .collect();
    if cleaned.chars().count() <= 120 {
        cleaned
    } else {
        let truncated: String = cleaned.chars().take(117).collect();
        format!("{}...", truncated)
    }
}

/// Emit an admin audit trace via `log::info!`. Thin wrapper around
/// [`format_admin_audit_line`] — the page-side call sites use this so
/// they never have to format the line themselves.
pub fn emit_admin_audit(
    target: &str,
    id: &str,
    action: &str,
    outcome: AdminAuditOutcome,
    note: Option<&str>,
) {
    log::info!(
        "{}",
        format_admin_audit_line(target, id, action, outcome, note)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_labels_are_stable() {
        assert_eq!(AdminAuditOutcome::Accepted.label(), "accepted");
        assert_eq!(AdminAuditOutcome::Rejected.label(), "rejected");
        assert_eq!(AdminAuditOutcome::NotWired.label(), "not_wired");
    }

    #[test]
    fn http_status_mapping_classifies_404_as_not_wired() {
        assert_eq!(
            AdminAuditOutcome::from_http_status(404),
            AdminAuditOutcome::NotWired
        );
        assert_eq!(
            AdminAuditOutcome::from_http_status(200),
            AdminAuditOutcome::Accepted
        );
        assert_eq!(
            AdminAuditOutcome::from_http_status(500),
            AdminAuditOutcome::Rejected
        );
        assert_eq!(
            AdminAuditOutcome::from_http_status(409),
            AdminAuditOutcome::Rejected
        );
    }

    #[test]
    fn audit_line_is_single_line_and_grep_friendly() {
        let line = format_admin_audit_line(
            "applet",
            "ap_01HXY",
            "approve",
            AdminAuditOutcome::Accepted,
            None,
        );
        assert!(!line.contains('\n'));
        assert!(line.contains("target=applet"));
        assert!(line.contains("id=ap_01HXY"));
        assert!(line.contains("action=approve"));
        assert!(line.contains("outcome=accepted"));
        assert!(!line.contains("note="));
    }

    #[test]
    fn audit_line_includes_note_when_present() {
        let line = format_admin_audit_line(
            "agent",
            "ag_01",
            "suspend",
            AdminAuditOutcome::Accepted,
            Some("temporary"),
        );
        assert!(line.contains("note=temporary"));
    }

    #[test]
    fn audit_line_drops_blank_note() {
        let line = format_admin_audit_line(
            "agent",
            "ag_01",
            "suspend",
            AdminAuditOutcome::Accepted,
            Some("   "),
        );
        assert!(!line.contains("note="));
    }

    #[test]
    fn audit_note_strips_newlines() {
        let line = format_admin_audit_line(
            "agent",
            "ag_01",
            "revoke",
            AdminAuditOutcome::Rejected,
            Some("line1\nline2\rline3\ttab"),
        );
        assert!(!line.contains('\n'));
        assert!(!line.contains('\r'));
        assert!(!line.contains('\t'));
        // spaces preserved as separators
        assert!(line.contains("line1 line2 line3 tab"));
    }

    #[test]
    fn audit_note_truncates_long_notes() {
        let long = "a".repeat(500);
        let line = format_admin_audit_line(
            "directory",
            "d_01",
            "reject",
            AdminAuditOutcome::Accepted,
            Some(&long),
        );
        // 117 + "..." == 120 chars after `note=`
        let after = line.split("note=").nth(1).unwrap();
        assert!(after.ends_with("..."));
        assert_eq!(after.chars().count(), 120);
    }
}
