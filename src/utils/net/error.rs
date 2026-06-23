use std::collections::BTreeMap;
use std::fmt;
use std::sync::OnceLock;

use cokret_core::error::{
    ERROR_CODE_AUTH_EXPIRED, ERROR_CODE_CAPABILITY_DENIED, ERROR_CODE_CAS_CONFLICT,
    ERROR_CODE_CONFLICT, ERROR_CODE_CURSOR_EXPIRED, ERROR_CODE_CURSOR_INVALID,
    ERROR_CODE_DUPLICATE_CONFLICT, ERROR_CODE_FAILED_PRECONDITION, ERROR_CODE_INVALID_PARAM,
    ERROR_CODE_NOT_FOUND, ERROR_CODE_POLICY_DENIED, ERROR_CODE_POLICY_VIOLATION,
    ERROR_CODE_RATE_LIMITED, ERROR_CODE_SCHEMA_VIOLATION, ERROR_CODE_SOFT_LOGGED_OUT,
    ERROR_CODE_TEMPORARILY_UNAVAILABLE, ERROR_CODE_UNAUTHENTICATED,
};
use regex_lite::Regex;
use serde::{Deserialize, Serialize};

/// Admin-side projection of the spec/SDK canonical error envelope
/// (`{ok, error:{code, message, retry_after_ms?, details?}, request_id}`,
/// see `cokret_core::models::api::ErrorEnvelope`). We keep a flattened local
/// shape — the wire envelope is parsed in [`from_wire`] — so the existing
/// call sites (`body.errcode`, `required_scope`) keep working while the
/// parse path reads the authoritative `error.code`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdminErrorEnvelope {
    /// The registry error code — sourced from `error.code` on the wire.
    /// Field name is kept as `errcode` for the local consumers; it is
    /// NOT a wire field name.
    pub errcode: String,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_after_ms: Option<u64>,
    /// D.1 — soland may attach the capability scope required for the
    /// failing action on 401/403 envelopes (e.g.
    /// `ck:scope:realm:01HXY/admin.write`). The canonical envelope nests
    /// this under `error.details.required_scope`; [`from_wire`] lifts it
    /// to this field for the UI.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_scope: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

impl AdminErrorEnvelope {
    /// Parse the canonical spec/SDK error envelope shape
    /// `{ok, error:{code, message, retry_after_ms?, details?}, request_id}`
    /// from an upstream (soland/coauth) error body. Returns `None` when the
    /// body is not a canonical envelope (e.g. an opaque HTML 502 from the
    /// proxy), so callers fall back to a status-only message.
    pub fn from_wire(text: &str) -> Option<Self> {
        let env: cokret_core::models::ErrorEnvelope = serde_json::from_str(text).ok()?;
        let required_scope = env
            .error
            .details
            .get("required_scope")
            .and_then(|v| v.as_str())
            .map(str::to_owned);
        Some(AdminErrorEnvelope {
            errcode: env.error.code,
            error: Some(env.error.message).filter(|m| !m.is_empty()),
            retry_after_ms: env.error.retry_after_ms,
            required_scope,
            extra: BTreeMap::new(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct HttpError {
    pub message: String,
    pub status: u16,
    pub body: Option<AdminErrorEnvelope>,
    pub request_id: Option<String>,
    pub retry_after_ms: Option<u64>,
}

impl HttpError {
    /// Construct an `HttpError` from a message with no HTTP status
    /// (status 0) and no optional metadata. For client-side / encode
    /// failures that never reached the server.
    pub fn message(msg: impl Into<String>) -> Self {
        HttpError {
            message: msg.into(),
            status: 0,
            body: None,
            request_id: None,
            retry_after_ms: None,
        }
    }

    /// Construct an `HttpError` with an explicit HTTP status and message,
    /// leaving all optional metadata unset. Public companion to
    /// [`HttpError::message`]; the live request path in `api/client.rs`
    /// builds errors with real `request_id`/`retry_after_ms` metadata, so
    /// this status-only constructor is currently exercised only by tests.
    #[allow(dead_code)]
    pub fn from_status(status: u16, msg: impl Into<String>) -> Self {
        HttpError {
            message: msg.into(),
            status,
            body: None,
            request_id: None,
            retry_after_ms: None,
        }
    }
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
    // P3A.8 — the soland reducer surfaces canonical CKP-0007 reason
    // strings (e.g. `circle_realm_mismatch`,
    // `circle_member_must_be_realm_member`) as the `errcode` field on
    // 422 responses. These six are the spec's
    // capability-action-registry reason codes; the SDK ships them as
    // public constants and the admin UI maps them straight to the
    // matching i18n key (`error.<reason>`). When an i18n string is
    // present we render it; otherwise we fall back to the in-line
    // English literal so brand-new codes still surface usefully.
    // Global report #10 (candidate 8) — the three CKP-0007 reason codes
    // the SDK ships as public constants are matched against
    // `cokret_core::error::REASON_CIRCLE_*` rather than hand-copied
    // literals, so a wire rename in the SDK breaks the build here. The
    // remaining three (`circle_already_terminal`,
    // `circle_capability_denied`, `circle_scope_rotation_in_progress`)
    // are sodmin/soland-local admin reasons not yet promoted to a core
    // constant, so they stay as literals.
    use cokret_core::error::{
        REASON_CIRCLE_MEMBER_MUST_BE_REALM_MEMBER, REASON_CIRCLE_NOT_ACTIVE,
        REASON_CIRCLE_REALM_MISMATCH,
    };
    let circle_reason: Option<&'static str> = match errcode {
        _ if errcode == REASON_CIRCLE_REALM_MISMATCH => Some("error.circle_realm_mismatch"),
        _ if errcode == REASON_CIRCLE_MEMBER_MUST_BE_REALM_MEMBER => {
            Some("error.circle_member_must_be_realm_member")
        }
        _ if errcode == REASON_CIRCLE_NOT_ACTIVE => Some("error.circle_not_active"),
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

    // soland's error envelope carries bare snake_case errcodes straight
    // from the spec error-code-registry (there is no `ck.error.*`
    // prefix in the registry, and `schema` is registered as
    // `schema_violation`). Match those literal registry codes only.
    let fallback = match errcode {
        ERROR_CODE_NOT_FOUND => "Resource not found",
        ERROR_CODE_UNAUTHENTICATED => "Authentication required",
        ERROR_CODE_CAPABILITY_DENIED => "Administrator capability denied",
        ERROR_CODE_RATE_LIMITED => "Rate limited",
        ERROR_CODE_TEMPORARILY_UNAVAILABLE => "Service temporarily unavailable",
        ERROR_CODE_SCHEMA_VIOLATION => "Request validation failed",
        ERROR_CODE_POLICY_DENIED | ERROR_CODE_POLICY_VIOLATION => {
            "Required policy approval is missing"
        }
        ERROR_CODE_AUTH_EXPIRED | ERROR_CODE_SOFT_LOGGED_OUT => "Session expired - sign in again",
        ERROR_CODE_DUPLICATE_CONFLICT | ERROR_CODE_CONFLICT => {
            "Request conflicted with a previous mutation"
        }
        ERROR_CODE_CAS_CONFLICT | ERROR_CODE_FAILED_PRECONDITION => {
            "Precondition failed - refresh and retry"
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

/// Format an admin-mutation error for the 404-tolerant pattern used by the
/// Stream H' actions (consent resolve, components refresh, covered_seals
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

pub fn should_reset_cursor_pagination(error: &HttpError, cursor: Option<&str>) -> bool {
    if cursor.filter(|c| !c.trim().is_empty()).is_none() {
        return false;
    }

    let errcode = error.body.as_ref().map(|body| body.errcode.as_str());
    // Registry codes are bare snake_case (no `ck.error.*` prefix exists in
    // the error-code-registry).
    matches!(
        (error.status, errcode),
        (410, Some(ERROR_CODE_CURSOR_EXPIRED))
            | (410, None)
            | (
                400,
                Some(ERROR_CODE_INVALID_PARAM | ERROR_CODE_CURSOR_INVALID)
            )
    )
}

#[cfg(test)]
mod tests {
    use super::{
        AdminErrorEnvelope, HttpError, display_error, format_optional_endpoint_error, redact_pii,
        should_reset_cursor_pagination,
    };

    fn err(status: u16, message: &str) -> HttpError {
        HttpError::from_status(status, message)
    }

    fn err_with_code(status: u16, errcode: &str) -> HttpError {
        HttpError {
            message: errcode.to_owned(),
            status,
            body: Some(AdminErrorEnvelope {
                errcode: errcode.to_owned(),
                ..Default::default()
            }),
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
            format_optional_endpoint_error("covered_seals advance", &e),
            "conflict"
        );
    }

    #[test]
    fn cursor_reset_only_for_active_cursor_errors() {
        assert!(should_reset_cursor_pagination(
            &err_with_code(410, "cursor_expired"),
            Some("ck:cursor:abc")
        ));
        assert!(should_reset_cursor_pagination(
            &err_with_code(400, "invalid_param"),
            Some("ck:cursor:abc")
        ));
        assert!(!should_reset_cursor_pagination(
            &err_with_code(400, "invalid_param"),
            None
        ));
        assert!(!should_reset_cursor_pagination(
            &err_with_code(500, "cursor_expired"),
            Some("ck:cursor:abc")
        ));
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
        let raw = "actor=urn:ck:actor:01HQX cursor=eyAB12";
        assert_eq!(redact_pii(raw), raw);
    }

    #[test]
    fn display_error_handles_protocol_codes() {
        let s = display_error("policy_denied", 412, "");
        assert!(s.contains("policy approval"));
        let s = display_error("auth_expired", 401, "");
        assert!(s.contains("Session expired"));
        let s = display_error("failed_precondition", 409, "");
        assert!(s.contains("Precondition failed"));
        // `schema_violation` is the registry code (not `schema`).
        let s = display_error("schema_violation", 422, "");
        assert!(s.contains("validation failed"));
    }

    #[test]
    fn display_error_redacts_raw_message() {
        let s = display_error("schema_violation", 400, "user bob@example.org rejected");
        assert!(!s.contains("bob@example.org"));
        assert!(s.contains("[email]"));
    }

    #[test]
    fn from_wire_reads_canonical_envelope_and_lifts_required_scope() {
        use super::AdminErrorEnvelope;
        // Canonical spec/SDK envelope: bare registry code under
        // `error.code`, required_scope nested in `error.details`.
        let raw = r#"{"ok":false,"error":{"code":"capability_denied","message":"denied","details":{"required_scope":"ck:scope:realm:01HXY/admin.write"}},"request_id":"req_1"}"#;
        let env = AdminErrorEnvelope::from_wire(raw).expect("parse");
        assert_eq!(env.errcode, "capability_denied");
        assert_eq!(
            env.required_scope.as_deref(),
            Some("ck:scope:realm:01HXY/admin.write")
        );
    }

    #[test]
    fn from_wire_reads_retry_after_and_omits_absent_scope() {
        use super::AdminErrorEnvelope;
        let raw = r#"{"ok":false,"error":{"code":"rate_limited","message":"slow down","retry_after_ms":2000},"request_id":"req_2"}"#;
        let env = AdminErrorEnvelope::from_wire(raw).expect("parse");
        assert_eq!(env.errcode, "rate_limited");
        assert_eq!(env.retry_after_ms, Some(2000));
        assert!(env.required_scope.is_none());
    }

    #[test]
    fn from_wire_returns_none_for_non_envelope_body() {
        use super::AdminErrorEnvelope;
        assert!(AdminErrorEnvelope::from_wire("<html>502</html>").is_none());
    }
}
