//! Audit admin surface — SDK-authoritative types plus thin display helpers.
//!
//! The row is the SDK `arkret_core::models::AdminAuditEntry` (D14
//! production projection, mirroring the durable audit record). AKP-0008
//! attribution (`executed_by` / `authorization_ref` / `actor_kind`) travels
//! inside `payload`; the helpers below surface it for the table columns.

pub use arkret_core::models::AdminAuditEntry;

/// Display helpers for the SDK [`AdminAuditEntry`].
pub trait AdminAuditEntryExt {
    fn payload_str(&self, key: &str) -> Option<String>;
    /// AKP-0008 — executing DID for agent-attributed envelopes.
    fn executed_by(&self) -> Option<String>;
    /// AKP-0008 — authorizing grant reference.
    fn authorization_ref(&self) -> Option<String>;
    /// AKP-0008 — actor classification stamped at admission.
    fn actor_kind(&self) -> Option<String>;
}

impl AdminAuditEntryExt for AdminAuditEntry {
    fn payload_str(&self, key: &str) -> Option<String> {
        self.payload
            .as_ref()
            .and_then(|payload| payload.get(key))
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned)
    }

    fn executed_by(&self) -> Option<String> {
        self.payload_str("executed_by")
    }

    fn authorization_ref(&self) -> Option<String> {
        self.payload_str("authorization_ref")
    }

    fn actor_kind(&self) -> Option<String> {
        self.payload_str("actor_kind")
    }
}
