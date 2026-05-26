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

/// Wire shape POSTed to `/api/admin/v1/audit/client-event`. The
/// envelope is intentionally schema-stable so soland's reducer can map
/// it straight to its audit row without a sodmin-specific adapter.
///
/// `target_type` + `target_id` mirror the local-only
/// `format_admin_audit_line` triple; `outcome` carries the mapped HTTP
/// status so the audit row distinguishes "soland accepted my click"
/// from "soland 404'd the endpoint" without re-deriving the
/// classification server-side.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AdminAuditClientEvent {
    pub target_type: String,
    pub target_id: String,
    pub action: String,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Build the wire payload for the `/api/admin/v1/audit/client-event`
/// POST. Pure helper — split out so we can unit-test the shape
/// without compiling the wasm fetch path.
pub fn build_client_event(
    target: &str,
    id: &str,
    action: &str,
    outcome: AdminAuditOutcome,
    note: Option<&str>,
) -> AdminAuditClientEvent {
    AdminAuditClientEvent {
        target_type: target.to_string(),
        target_id: id.to_string(),
        action: action.to_string(),
        outcome: outcome.label().to_string(),
        note: note
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string()),
    }
}

/// Server-side admin audit POST. Mirrors [`emit_admin_audit`] but
/// additionally fires a best-effort `POST
/// /api/admin/v1/audit/client-event` so the audit row also lands in
/// the soland audit feed (not only the browser console).
///
/// 404 / 5xx is intentionally tolerated — soland may not have wired
/// the client-event sink yet, and a missing audit-of-the-click should
/// never break the actual click. We swallow the error and log a
/// single line so the operator can see the POST happened even when
/// the route is missing.
#[cfg(target_arch = "wasm32")]
pub fn emit_admin_audit_server(
    target: &str,
    id: &str,
    action: &str,
    outcome: AdminAuditOutcome,
    note: Option<&str>,
) {
    // Local console line first so the breadcrumb always appears even
    // if the network POST is queued / fails.
    emit_admin_audit(target, id, action, outcome, note);

    let payload = build_client_event(target, id, action, outcome, note);
    let body = match serde_json::to_string(&payload) {
        Ok(b) => b,
        Err(e) => {
            log::warn!("sodmin.admin.audit_server serialize failed: {e}");
            return;
        }
    };

    dioxus::prelude::spawn(async move {
        let res: Result<serde_json::Value, _> =
            crate::api::client::api_client("/api/admin/v1/audit/client-event", "POST", Some(body))
                .await;
        if let Err(e) = res {
            // Don't toast — this is a fire-and-forget breadcrumb. Just
            // surface in the console for the operator who's actively
            // debugging audit wiring.
            log::warn!(
                "sodmin.admin.audit_server POST failed status={} err={}",
                e.status,
                e.message
            );
        }
    });
}

/// Non-wasm fallback for tests and host-side compilation. The actual
/// POST is wasm-only; this keeps `cargo check` on a host target from
/// dragging in the gloo-net dependency in test mode.
#[cfg(not(target_arch = "wasm32"))]
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
    fn client_event_payload_carries_outcome_label() {
        let ev = build_client_event(
            "agent",
            "ag_01",
            "approve",
            AdminAuditOutcome::Accepted,
            None,
        );
        assert_eq!(ev.target_type, "agent");
        assert_eq!(ev.target_id, "ag_01");
        assert_eq!(ev.action, "approve");
        assert_eq!(ev.outcome, "accepted");
        assert!(ev.note.is_none());
    }

    #[test]
    fn client_event_drops_blank_note() {
        let ev = build_client_event(
            "agent",
            "ag_01",
            "suspend",
            AdminAuditOutcome::Rejected,
            Some("   "),
        );
        assert!(ev.note.is_none());
    }

    #[test]
    fn client_event_serializes_to_stable_json() {
        let ev = build_client_event(
            "applet",
            "ap_01",
            "revoke",
            AdminAuditOutcome::NotWired,
            Some("hand-off to coauth"),
        );
        let json = serde_json::to_string(&ev).expect("json");
        assert!(json.contains("\"target_type\":\"applet\""));
        assert!(json.contains("\"target_id\":\"ap_01\""));
        assert!(json.contains("\"action\":\"revoke\""));
        assert!(json.contains("\"outcome\":\"not_wired\""));
        assert!(json.contains("\"note\":\"hand-off to coauth\""));
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
