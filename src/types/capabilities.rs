//! DTO shapes for the capability grant admin surface.

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

pub const CAPABILITY_GRANT_SCHEMA: &str = "ck.schema.capability.v1";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilityGrant {
    #[serde(default)]
    pub id: String,
    #[serde(default = "capability_schema")]
    pub schema: String,
    #[serde(default, alias = "grantor_id")]
    pub issuer: String,
    #[serde(default, alias = "grantee_id")]
    pub subject: String,
    #[serde(default, alias = "capability", deserialize_with = "string_or_vec")]
    pub actions: Vec<String>,
    #[serde(default, alias = "scope", deserialize_with = "resources_from_wire")]
    pub resources: Vec<serde_json::Value>,
    #[serde(default, deserialize_with = "constraints_from_wire")]
    pub constraints: Vec<serde_json::Value>,
    #[serde(default)]
    pub parent_grant_id: Option<String>,
    #[serde(default, alias = "granted_at")]
    pub issued_at: Option<String>,
    #[serde(default)]
    pub not_before: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub revoked_at: Option<String>,
    #[serde(default)]
    pub revoked_by: Option<String>,
    #[serde(default, alias = "is_revoked", skip_serializing)]
    pub lifecycle_revoked: bool,
    #[serde(default)]
    pub proofs: Vec<serde_json::Value>,
}

impl CapabilityGrant {
    pub fn is_revoked(&self) -> bool {
        self.lifecycle_revoked || self.revoked_at.is_some()
    }

    pub fn actions_display(&self) -> String {
        if self.actions.is_empty() {
            "-".to_string()
        } else {
            self.actions.join(", ")
        }
    }

    pub fn resources_display(&self) -> String {
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GrantCapabilityRequest {
    #[serde(default = "capability_schema")]
    pub schema: String,
    pub subject: String,
    pub actions: Vec<String>,
    pub resources: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_grant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

pub fn capability_schema() -> String {
    CAPABILITY_GRANT_SCHEMA.to_string()
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
    if trimmed.starts_with("ck:flow:") {
        return vec![serde_json::json!({ "kind": "flow", "flow_id": trimmed })];
    }
    if trimmed.starts_with("ck:message:") {
        return vec![serde_json::json!({ "kind": "message", "message_id": trimmed })];
    }
    if trimmed.starts_with("did:") {
        return vec![serde_json::json!({ "kind": "actor", "actor_id": trimmed })];
    }
    vec![serde_json::json!({ "kind": "*", "selector": trimmed })]
}

fn string_or_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(Vec::new()),
        Some(serde_json::Value::String(s)) => Ok(if s.trim().is_empty() {
            Vec::new()
        } else {
            vec![s]
        }),
        Some(serde_json::Value::Array(items)) => Ok(items
            .into_iter()
            .filter_map(|item| item.as_str().map(ToOwned::to_owned))
            .collect()),
        Some(other) => Err(de::Error::custom(format!(
            "expected action string or array, got {other}"
        ))),
    }
}

fn resources_from_wire<'de, D>(deserializer: D) -> Result<Vec<serde_json::Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(Vec::new()),
        Some(serde_json::Value::String(scope)) => Ok(if scope.trim().is_empty() {
            Vec::new()
        } else {
            vec![serde_json::json!({ "kind": "*", "legacy_scope": scope })]
        }),
        Some(serde_json::Value::Array(items)) => Ok(items),
        Some(serde_json::Value::Object(map)) => Ok(vec![serde_json::Value::Object(map)]),
        Some(other) => Err(de::Error::custom(format!(
            "expected resource selector array, got {other}"
        ))),
    }
}

fn constraints_from_wire<'de, D>(deserializer: D) -> Result<Vec<serde_json::Value>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(Vec::new()),
        Some(serde_json::Value::Array(items)) => Ok(items),
        Some(serde_json::Value::Object(map)) => Ok(vec![serde_json::Value::Object(map)]),
        Some(other) => Err(de::Error::custom(format!(
            "expected constraint object or array, got {other}"
        ))),
    }
}

fn display_resource_selector(value: &serde_json::Value) -> String {
    if let Some(scope) = value
        .get("legacy_scope")
        .and_then(serde_json::Value::as_str)
    {
        return scope.to_string();
    }
    if value == &serde_json::json!({ "kind": "*" }) {
        return "*".to_string();
    }
    serde_json::to_string(value).unwrap_or_else(|_| "-".to_string())
}

/// PATCH body for `/_soland/admin/capabilities/{id}` - fine-grained
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
    use super::{
        CAPABILITY_GRANT_SCHEMA, CapabilityGrant, GrantCapabilityRequest, UpdateCapabilityRequest,
        capability_resources_from_input,
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
    fn capability_grant_reads_legacy_response_without_emitting_legacy_fields() {
        let grant: CapabilityGrant = serde_json::from_value(serde_json::json!({
            "id": "ck:grant:01964137-0000-7000-8000-000000000001",
            "grantor_id": "did:web:issuer.example",
            "grantee_id": "did:web:subject.example",
            "capability": "ck.message.create",
            "scope": "ck:realm:01964137-0000-7000-8000-000000000001",
            "granted_at": "2026-06-07T00:00:00Z",
            "is_revoked": true
        }))
        .expect("legacy response should deserialize");

        assert_eq!(grant.issuer, "did:web:issuer.example");
        assert_eq!(grant.subject, "did:web:subject.example");
        assert_eq!(grant.actions, vec!["ck.message.create"]);
        assert!(grant.is_revoked());
        assert_eq!(grant.issued_at.as_deref(), Some("2026-06-07T00:00:00Z"));

        let value = serde_json::to_value(&grant).expect("serializes");
        assert!(value.get("grantor_id").is_none());
        assert!(value.get("grantee_id").is_none());
        assert!(value.get("capability").is_none());
        assert!(value.get("scope").is_none());
        assert!(value.get("is_revoked").is_none());
    }
}
