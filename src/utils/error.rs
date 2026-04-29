use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

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
        _ => message,
    };
    if message.is_empty() || fallback == message {
        format!("{errcode} ({status}): {fallback}")
    } else {
        format!("{errcode} ({status}): {fallback}: {message}")
    }
}

/// Format an error message for display in toasts, including the request ID if available.
pub fn format_error_with_ref(error: &HttpError) -> String {
    error.to_string()
}
