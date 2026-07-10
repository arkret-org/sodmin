//! DTO shapes for the handle management admin surface.

use serde::{Deserialize, Serialize};

// ── Handle availability ──

// ── Handle management (T6.2 §2) ──

/// One row in `GET /_soland/admin/handles`. Mirrors the `ak.handle.*` cell
/// projection — `canonical_uri` is the cell subject, `aliases` is the
/// projected handle set, `issuer_did` is the principal that signed the
/// most recent assignment Control Move.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandleRecord {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub canonical_uri: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub issuer_did: Option<String>,
    #[serde(default)]
    pub subject_id: Option<String>,
    #[serde(default)]
    pub assigned_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub last_reassignment_at: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

/// Single audit event for `GET /_soland/admin/handles/{id}/audit`. The
/// audit table is what T3.2 created — we surface the minimum the
/// operator needs to triage.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandleAuditEvent {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub action: String,
    #[serde(default)]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub previous_subject_id: Option<String>,
    #[serde(default)]
    pub new_subject_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HandleReassignRequest {
    pub new_subject_id: String,
    pub reason: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::HandleRecord;

    #[test]
    fn handle_record_round_trip() {
        let record: HandleRecord = serde_json::from_value(json!({
            "id": "h-1",
            "canonical_uri": "ak:handle:@alice",
            "aliases": ["@alice", "@alice.example"],
            "issuer_did": "did:web:auth.example.com",
            "subject_id": "did:key:zABC",
            "status": "active"
        }))
        .expect("handle record should deserialize");

        assert_eq!(record.canonical_uri, "ak:handle:@alice");
        assert_eq!(record.aliases.len(), 2);
        assert_eq!(record.status.as_deref(), Some("active"));
    }
}
