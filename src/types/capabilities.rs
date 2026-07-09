//! Capability grant admin surface — SDK-authoritative types plus thin
//! display helpers.
//!
//! The grant itself is the SDK `arkret_core::models::CapabilityGrant`
//! (`GrantId` / `Did` / `DateTime<Utc>` strong types, field set per
//! `ck.schema.capability.v1`). sodmin adds no wire mirror or field aliases;
//! display-only conveniences live in
//! [`CapabilityGrantExt`].

pub use arkret_core::models::{CapabilityGrant, CapabilitySubject};

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

fn display_resource_selector(value: &serde_json::Value) -> String {
    if value == &serde_json::json!({ "kind": "*" }) {
        return "*".to_string();
    }
    serde_json::to_string(value).unwrap_or_else(|_| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::{CapabilityGrant, CapabilityGrantExt};

    const CAPABILITY_GRANT_SCHEMA: &str = "ck.schema.capability.v1";

    #[test]
    fn sdk_capability_grant_parses_spec_wire_shape() {
        let grant: CapabilityGrant = serde_json::from_value(serde_json::json!({
            "id": "ak:grant:01964137-0000-7000-8000-000000000001",
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
