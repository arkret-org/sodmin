//! Capability grant admin surface — SDK-authoritative types plus thin
//! display helpers.
//!
//! The grant itself is the SDK `cokret_core::model::CapabilityGrant`
//! (`GrantId` / `Did` / `DateTime<Utc>` strong types, field set per
//! `ck.schema.capability.v1`). sodmin adds no wire mirror and no legacy
//! field aliases; display-only conveniences live in
//! [`CapabilityGrantExt`].

pub use cokret_core::model::{CapabilityGrant, CapabilitySubject};
use serde::Serialize;

pub const CAPABILITY_GRANT_SCHEMA: &str = "ck.schema.capability.v1";

/// Display helpers for the SDK [`CapabilityGrant`].
pub trait CapabilityGrantExt {
    fn is_revoked(&self) -> bool;
    fn subject_display(&self) -> String;
    fn actions_display(&self) -> String;
    fn resources_display(&self) -> String;
}

impl CapabilityGrantExt for CapabilityGrant {
    fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    fn subject_display(&self) -> String {
        match &self.subject {
            CapabilitySubject::Did(did) => did.to_string(),
            CapabilitySubject::Selector(value) => value.to_string(),
        }
    }

    fn actions_display(&self) -> String {
        if self.actions.is_empty() {
            "-".to_string()
        } else {
            self.actions.join(", ")
        }
    }

    fn resources_display(&self) -> String {
        if self.resources.is_empty() {
            return "-".to_string();
        }
        self.resources
            .iter()
            .map(display_resource_selector)
            .collect::<Vec<_>>()
            .join(", ")
    }
}

/// POST body for `/_soland/admin/capabilities` — spec-shaped
/// (`ck.schema.capability.v1` field names; no legacy aliases).
#[derive(Debug, Clone, Serialize, Default)]
pub struct GrantCapabilityRequest {
    pub schema: String,
    pub subject: String,
    pub actions: Vec<String>,
    pub resources: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_grant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

pub fn capability_resources_from_input(input: &str) -> Vec<serde_json::Value> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return vec![serde_json::json!({ "kind": "*" })];
    }
    if trimmed.starts_with("ck:realm:") {
        return vec![serde_json::json!({ "kind": "realm", "realm_id": trimmed })];
    }
    if trimmed.starts_with("ck:space:") {
        return vec![serde_json::json!({ "kind": "space", "space_id": trimmed })];
    }
    if trimmed.starts_with("ck:circle:") {
        return vec![serde_json::json!({ "kind": "circle", "circle_id": trimmed })];
    }
    if trimmed.starts_with("ck:strand:") {
        return vec![serde_json::json!({ "kind": "strand", "strand_id": trimmed })];
    }
    if trimmed.starts_with("ck:message:") {
        return vec![serde_json::json!({ "kind": "message", "message_id": trimmed })];
    }
    if trimmed.starts_with("did:") {
        return vec![serde_json::json!({ "kind": "actor", "actor_id": trimmed })];
    }
    vec![serde_json::json!({ "kind": "*", "selector": trimmed })]
}

fn display_resource_selector(value: &serde_json::Value) -> String {
    if value == &serde_json::json!({ "kind": "*" }) {
        return "*".to_string();
    }
    serde_json::to_string(value).unwrap_or_else(|_| "-".to_string())
}

/// PATCH body for `/_soland/admin/capabilities/{id}` - fine-grained
/// edits to an existing grant's constraints. All fields optional; the
/// admin only sends the keys that actually changed.
#[derive(Debug, Clone, Serialize, Default)]
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
    use super::{
        CAPABILITY_GRANT_SCHEMA, CapabilityGrant, CapabilityGrantExt, GrantCapabilityRequest,
        UpdateCapabilityRequest, capability_resources_from_input,
    };

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

    #[test]
    fn grant_request_serializes_spec_fields_only() {
        let req = GrantCapabilityRequest {
            schema: CAPABILITY_GRANT_SCHEMA.to_string(),
            subject: "did:web:alice.example".to_string(),
            actions: vec!["ck.circle.manage".to_string()],
            resources: capability_resources_from_input(
                "ck:circle:01964137-0000-7000-8000-000000000001",
            ),
            constraints: Vec::new(),
            parent_grant_id: None,
            not_before: None,
            expires_at: Some("2027-01-01T00:00:00Z".to_string()),
        };
        let value = serde_json::to_value(&req).expect("serializes");
        assert_eq!(value["schema"], CAPABILITY_GRANT_SCHEMA);
        assert_eq!(value["subject"], "did:web:alice.example");
        assert_eq!(value["actions"][0], "ck.circle.manage");
        assert_eq!(value["resources"][0]["kind"], "circle");
        assert!(value.get("grantee_id").is_none());
        assert!(value.get("capability").is_none());
        assert!(value.get("scope").is_none());
    }

    #[test]
    fn sdk_capability_grant_parses_spec_wire_shape() {
        let grant: CapabilityGrant = serde_json::from_value(serde_json::json!({
            "id": "ck:grant:01964137-0000-7000-8000-000000000001",
            "schema": CAPABILITY_GRANT_SCHEMA,
            "issuer": "did:web:issuer.example",
            "subject": "did:web:subject.example",
            "actions": ["ck.message.create"],
            "resources": [{ "kind": "*" }],
            "issued_at": "2026-06-07T00:00:00Z",
            "proofs": []
        }))
        .expect("spec-shaped grant should deserialize");

        assert_eq!(grant.issuer.as_str(), "did:web:issuer.example");
        assert_eq!(grant.subject_display(), "did:web:subject.example");
        assert_eq!(grant.actions_display(), "ck.message.create");
        assert_eq!(grant.resources_display(), "*");
        assert!(!grant.is_revoked());
    }
}
