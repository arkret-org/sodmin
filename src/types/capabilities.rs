//! DTO shapes for the capability grant admin surface.

use serde::{Deserialize, Serialize};

// ── Capability types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilityGrant {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub grantor_id: String,
    #[serde(default)]
    pub grantee_id: String,
    #[serde(default)]
    pub capability: String,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub constraints: Option<serde_json::Value>,
    #[serde(default)]
    pub granted_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub is_revoked: bool,
    #[serde(default)]
    pub delegation_depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GrantCapabilityRequest {
    pub grantee_id: String,
    pub capability: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constraints: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// PATCH body for `/_soland/admin/capabilities/{id}` — fine-grained
/// edits to an existing grant's constraints. All fields optional; the
/// admin only sends the keys that actually changed.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateCapabilityRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_write_fields: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub facets_allow: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approval_required: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::UpdateCapabilityRequest;

    #[test]
    fn update_capability_request_serializes_only_set_fields() {
        let req = UpdateCapabilityRequest {
            expires_at: Some("2027-01-01T00:00:00Z".to_string()),
            allowed_write_fields: Some(vec!["body.text".to_string()]),
            facets_allow: None,
            approval_required: Some(true),
        };
        let serialized = serde_json::to_string(&req).expect("serializes");
        assert!(serialized.contains("expires_at"));
        assert!(serialized.contains("allowed_write_fields"));
        assert!(serialized.contains("approval_required"));
        assert!(!serialized.contains("facets_allow"));
    }
}
