use regex_lite::Regex;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;
use std::sync::OnceLock;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdminErrorEnvelope {
    pub errcode: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone)]
pub struct HttpError {
    pub message: String,
    pub status: u16,
    pub body: Option<AdminErrorEnvelope>,
    pub request_id: Option<String>,
    pub retry_after_ms: Option<u64>,
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        if let Some(ref rid) = self.request_id {
            write!(f, " (ref: {rid})")?;
        }
        if let Some(retry_after_ms) = self.retry_after_ms {
            write!(f, " (retry after: {}s)", retry_after_ms / 1000)?;
        }
        Ok(())
    }
}

impl std::error::Error for HttpError {}

pub fn display_error(errcode: &str, status: u16, message: &str) -> String {
    // P3A.8 — the soland reducer surfaces canonical CXP-0007 reason
    // strings (e.g. `circle_realm_mismatch`,
    // `circle_member_must_be_realm_member`) as the `errcode` field on
    // 422 responses. These six are the spec's
    // capability-action-registry reason codes; the SDK ships them as
    // public constants and the admin UI maps them straight to the
    // matching i18n key (`error.<reason>`). When an i18n string is
    // present we render it; otherwise we fall back to the in-line
    // English literal so brand-new codes still surface usefully.
    let circle_reason: Option<&'static str> = match errcode {
        "circle_realm_mismatch" => Some("error.circle_realm_mismatch"),
        "circle_member_must_be_realm_member" => Some("error.circle_member_must_be_realm_member"),
        "circle_not_active" => Some("error.circle_not_active"),
        "circle_already_terminal" => Some("error.circle_already_terminal"),
        "circle_capability_denied" => Some("error.circle_capability_denied"),
        "circle_scope_rotation_in_progress" => Some("error.circle_scope_rotation_in_progress"),
        _ => None,
    };
    if let Some(key) = circle_reason {
        let localised = crate::utils::i18n::t(key);
        let safe_message = redact_pii(message);
        if safe_message.is_empty() || localised.contains(&safe_message) {
            return format!("{errcode} ({status}): {localised}");
        }
        return format!("{errcode} ({status}): {localised}: {safe_message}");
    }

    let fallback = match errcode {
        "cx.error.not_found" | "not_found" => "Resource not found",
        "cx.error.unauthenticated" | "unauthenticated" => "Authentication required",
        "cx.error.capability_denied" | "capability_denied" => "Administrator capability denied",
        "cx.error.rate_limited" | "rate_limited" => "Rate limited",
        "cx.error.temporarily_unavailable" | "temporarily_unavailable" => {
            "Service temporarily unavailable"
        }
        "cx.error.validation" | "cx.error.schema" | "validation" | "schema" => {
            "Request validation failed"
        }
        "cx.error.recovery_required" | "recovery_required" => {
            "Recovery flow must complete before this action is allowed"
        }
        "cx.error.policy_required" | "policy_required" => "Required policy approval is missing",
        "cx.error.session_expired" | "session_expired" => "Session expired — sign in again",
        "cx.error.idempotency_conflict" | "idempotency_conflict" => {
            "Idempotency key conflicted with a previous request"
        }
        "cx.error.precondition_failed" | "precondition_failed" => {
            "Precondition failed — refresh and retry"
        }
        _ => message,
    };
    let safe_message = redact_pii(message);
    if safe_message.is_empty() || fallback == safe_message {
        format!("{errcode} ({status}): {fallback}")
    } else {
        format!("{errcode} ({status}): {fallback}: {safe_message}")
    }
}

/// Strip obvious PII (emails, IP addresses, bearer tokens) from server
/// error text before it lands in a toast. Server messages are best-effort
/// human-readable and routinely include the offending input — emitting
/// the user's email or IP into the admin UI is a leak.
pub fn redact_pii(message: &str) -> String {
    static PATTERNS: OnceLock<[(Regex, &'static str); 4]> = OnceLock::new();
    let patterns = PATTERNS.get_or_init(|| {
        [
            (
                // Email: covers most RFC 5322 atoms used in practice.
                Regex::new(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}").unwrap(),
                "[email]",
            ),
            (
                // IPv4 literal.
                Regex::new(r"\b(?:\d{1,3}\.){3}\d{1,3}\b").unwrap(),
                "[ip]",
            ),
            (
                // Bearer / authorization tokens that occasionally leak
                // through server-side validation errors.
                Regex::new(r"(?i)bearer\s+[A-Za-z0-9._\-]+").unwrap(),
                "[token]",
            ),
            (
                // JWT-shaped triple (header.payload.signature). Distinct
                // from plain UUIDs/IDs which lack the dot structure, so
                // diagnostic identifiers stay readable.
                Regex::new(r"\b[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{8,}\.[A-Za-z0-9_\-]{4,}\b")
                    .unwrap(),
                "[jwt]",
            ),
        ]
    });

    let mut out = message.to_string();
    for (re, replacement) in patterns {
        out = re.replace_all(&out, *replacement).into_owned();
    }
    out
}

/// Format an error message for display in toasts, including the request ID if available.
pub fn format_error_with_ref(error: &HttpError) -> String {
    error.to_string()
}

/// Format an admin-mutation error for the 404-tolerant pattern used by the
/// Stream H' actions (consent resolve, components refresh, covered_frontier
/// advance). When the backend hasn't wired the route yet, soland returns
/// 404, and the admin sees a clearer "endpoint not yet wired" message
/// rather than a generic "Resource not found". For non-404 errors the
/// regular formatted message is returned unchanged.
///
/// Pure helper — no I/O — so it can be unit-tested without hitting the
/// network. The action label is included in the not-yet-wired toast so
/// operators know which feature is missing on the backend (e.g. "consent
/// resolve endpoint not yet wired").
pub fn format_optional_endpoint_error(action: &str, error: &HttpError) -> String {
    if error.status == 404 {
        format!("{action} endpoint not yet wired")
    } else {
        error.message.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::{HttpError, display_error, format_optional_endpoint_error, redact_pii};

    fn err(status: u16, message: &str) -> HttpError {
        HttpError {
            message: message.to_string(),
            status,
            body: None,
            request_id: None,
            retry_after_ms: None,
        }
    }

    #[test]
    fn optional_endpoint_404_shows_not_yet_wired() {
        let e = err(404, "Not Found");
        let msg = format_optional_endpoint_error("consent resolve", &e);
        assert!(msg.to_lowercase().contains("not yet wired"));
        assert!(msg.contains("consent resolve"));
    }

    #[test]
    fn optional_endpoint_non_404_returns_original_message() {
        let e = err(500, "boom");
        assert_eq!(
            format_optional_endpoint_error("components refresh", &e),
            "boom"
        );
        let e = err(409, "conflict");
        assert_eq!(
            format_optional_endpoint_error("covered_frontier advance", &e),
            "conflict"
        );
    }

    #[test]
    fn redact_pii_strips_email_ip_token() {
        let raw = "denied for alice@example.com from 10.0.0.7 with Bearer abc.def-123";
        let safe = redact_pii(raw);
        assert!(!safe.contains("alice@example.com"));
        assert!(!safe.contains("10.0.0.7"));
        assert!(!safe.contains("abc.def-123"));
        assert!(safe.contains("[email]"));
        assert!(safe.contains("[ip]"));
        assert!(safe.contains("[token]"));
    }

    #[test]
    fn redact_pii_preserves_diagnostic_ids() {
        // UUIDs and short cursor tokens stay readable so admins can grep
        // logs.
        let raw = "actor=urn:cx:actor:01HQX cursor=eyAB12";
        assert_eq!(redact_pii(raw), raw);
    }

    #[test]
    fn display_error_handles_protocol_codes() {
        let s = display_error("cx.error.recovery_required", 412, "");
        assert!(s.contains("Recovery flow"));
        let s = display_error("cx.error.policy_required", 412, "");
        assert!(s.contains("policy approval"));
        let s = display_error("cx.error.session_expired", 401, "");
        assert!(s.contains("Session expired"));
    }

    #[test]
    fn display_error_redacts_raw_message() {
        let s = display_error("cx.error.validation", 400, "user bob@example.org rejected");
        assert!(!s.contains("bob@example.org"));
        assert!(s.contains("[email]"));
    }
}
