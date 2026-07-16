//! Client-side admin audit breadcrumb
//!
//! Why this exists: per-row destructive mutations on the admin pages
//! leave a structured breadcrumb in the browser console + any attached
//! log sink, so an operator can correlate a sodmin click with the
//! soland-side audit row it produced.
//!
//! This is a console-only correlation aid ("did the click actually
//! fire"), never the audit of record. The authoritative row is appended
//! *server-side* by soland while it handles the admin action itself —
//! see `routing/admin/spec.rs` (`admin_account_state_action` →
//! `append_audit_log`) and `routing/admin/actors.rs` — stamped with the
//! authenticated session actor rather than a client-asserted one, and
//! carrying the target id. sodmin therefore does not, and must not,
//! post its own copy: `/_soland/self/audit/*` is the *client
//! self-service* telemetry sink (it binds writes to the posting actor's
//! own session and has no target field), not an admin audit surface.
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
/// means it returned a non-2xx.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdminAuditOutcome {
    Accepted,
    Rejected,
}

impl AdminAuditOutcome {
    pub fn label(&self) -> &'static str {
        match self {
            AdminAuditOutcome::Accepted => "accepted",
            AdminAuditOutcome::Rejected => "rejected",
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

/// Record an admin click that soland has already audited server-side.
///
/// Thin alias for [`emit_admin_audit`], kept as its own name so the call
/// sites read as "this mutation is on the audit trail" — the trail being
/// soland's `append_audit_log` row for the admin endpoint that was just
/// called, which this line only correlates with.
pub fn emit_admin_audit_server(
    target: &str,
    id: &str,
    action: &str,
    outcome: AdminAuditOutcome,
    note: Option<&str>,
) {
    emit_admin_audit(target, id, action, outcome, note);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcome_labels_are_stable() {
        assert_eq!(AdminAuditOutcome::Accepted.label(), "accepted");
        assert_eq!(AdminAuditOutcome::Rejected.label(), "rejected");
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
