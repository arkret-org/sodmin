//! Admin DTOs for the soland Realm policy editor.
//!
//! Mirrors the `ck.component.realm.policy.v1` component body. Submit
//! constructs a cas-register Control Move via
//! `POST /_soland/admin/realms/{realm_id}/policy` with a typed body the
//! backend wraps into a Control Move + signature.

use serde::{Deserialize, Serialize};

/// Components inside a Realm policy. Each lives at a known component
/// state-key; the editor exposes them in a flat form so the operator
/// edits the whole policy as one transaction.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealmPolicy {
    /// History visibility for unauthenticated joiners. One of:
    /// `joined` / `invited` / `shared` / `world_readable`.
    #[serde(default)]
    pub history_visibility: String,
    /// Join rule — `public` / `invite` / `restricted` / `knock`.
    #[serde(default)]
    pub join_rule: String,
    /// Guest access — `can_join` / `forbidden`.
    #[serde(default)]
    pub guest_access: String,
    /// Federate the Realm at all. `false` = same-server only.
    #[serde(default = "default_federate")]
    pub federate: bool,
    /// Encryption algorithm — empty string = no E2EE.
    #[serde(default)]
    pub encryption_algorithm: String,
    /// `ck.realm.disappearing_policy` cell payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disappearing_policy: Option<RealmDisappearingPolicy>,
    /// `ck.realm.search_policy` cell payload.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub search_policy: Option<RealmSearchPolicy>,
}

fn default_federate() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RealmDisappearingPolicy {
    pub enabled: bool,
    pub max_ttl_ms: u64,
    pub allowed_triggers: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_grace_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub allow_plaintext_realms: Option<bool>,
}

impl Default for RealmDisappearingPolicy {
    fn default() -> Self {
        Self {
            enabled: false,
            max_ttl_ms: 86_400_000,
            allowed_triggers: vec!["on_send".to_owned()],
            default_grace_ms: Some(0),
            allow_plaintext_realms: Some(false),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RealmSearchPolicy {
    pub enabled_profile_refs: Vec<String>,
    pub allowed_service_dids: Vec<String>,
    pub data_classes: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index_retention_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revocation_behavior: Option<String>,
}

impl RealmPolicy {
    /// Validate that the policy is internally consistent. Returns the
    /// first invariant violation as a human-readable string. Used by the
    /// editor before posting the cas-register Control Move.
    pub fn validate(&self) -> Result<(), String> {
        if self.history_visibility.is_empty() {
            return Err("history_visibility is required".into());
        }
        if !matches!(
            self.history_visibility.as_str(),
            "joined" | "invited" | "shared" | "world_readable"
        ) {
            return Err(format!(
                "history_visibility must be one of joined / invited / shared / world_readable (got `{}`)",
                self.history_visibility,
            ));
        }
        if self.join_rule.is_empty() {
            return Err("join_rule is required".into());
        }
        if !matches!(
            self.join_rule.as_str(),
            "public" | "invite" | "restricted" | "knock"
        ) {
            return Err(format!(
                "join_rule must be one of public / invite / restricted / knock (got `{}`)",
                self.join_rule,
            ));
        }
        if !self.guest_access.is_empty()
            && !matches!(self.guest_access.as_str(), "can_join" | "forbidden")
        {
            return Err(format!(
                "guest_access must be can_join or forbidden (got `{}`)",
                self.guest_access,
            ));
        }
        // Encryption algorithm — empty = no E2EE — anything else must be
        // one of the known MLS variants. We keep this lenient because
        // the SDK adds new variants over time; the backend is the
        // ultimate gate.
        if let Some(policy) = &self.disappearing_policy {
            policy.validate()?;
        }
        if let Some(policy) = &self.search_policy {
            policy.validate()?;
        }
        Ok(())
    }
}

impl RealmDisappearingPolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.max_ttl_ms == 0 {
            return Err("disappearing_policy.max_ttl_ms must be >= 1".into());
        }
        if self.allowed_triggers.is_empty() {
            return Err("disappearing_policy.allowed_triggers is required".into());
        }
        for trigger in &self.allowed_triggers {
            if !matches!(
                trigger.as_str(),
                "on_send" | "on_first_read" | "on_last_read"
            ) {
                return Err(format!(
                    "disappearing_policy.allowed_triggers contains unknown trigger `{trigger}`"
                ));
            }
        }
        ensure_unique(
            &self.allowed_triggers,
            "disappearing_policy.allowed_triggers",
        )?;
        Ok(())
    }
}

impl RealmSearchPolicy {
    pub fn validate(&self) -> Result<(), String> {
        for profile in &self.enabled_profile_refs {
            if !matches!(
                profile.as_str(),
                "ck.profile.search.client_index.v1" | "ck.profile.search.blind_index.v1"
            ) {
                return Err(format!(
                    "search_policy.enabled_profile_refs contains unknown profile `{profile}`"
                ));
            }
        }
        ensure_unique(
            &self.enabled_profile_refs,
            "search_policy.enabled_profile_refs",
        )?;
        for did in &self.allowed_service_dids {
            if !did.starts_with("did:") || did.chars().any(char::is_whitespace) {
                return Err(format!(
                    "search_policy.allowed_service_dids contains invalid DID `{did}`"
                ));
            }
        }
        ensure_unique(
            &self.allowed_service_dids,
            "search_policy.allowed_service_dids",
        )?;
        for data_class in &self.data_classes {
            if !matches!(
                data_class.as_str(),
                "encrypted_index" | "blind_tokens" | "plaintext" | "reversible_summary"
            ) {
                return Err(format!(
                    "search_policy.data_classes contains unknown data class `{data_class}`"
                ));
            }
        }
        ensure_unique(&self.data_classes, "search_policy.data_classes")?;
        if let Some(behavior) = &self.revocation_behavior
            && !matches!(behavior.as_str(), "fail_closed" | "drop_stale")
        {
            return Err(format!(
                "search_policy.revocation_behavior must be fail_closed or drop_stale (got `{behavior}`)"
            ));
        }
        Ok(())
    }
}

fn ensure_unique(values: &[String], field: &str) -> Result<(), String> {
    for (index, value) in values.iter().enumerate() {
        if values[..index].contains(value) {
            return Err(format!("{field} contains duplicate value `{value}`"));
        }
    }
    Ok(())
}

/// Body `POSTed` to `/_soland/admin/realms/{realm_id}/policy`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateRealmPolicyRequest {
    pub policy: RealmPolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_policy() -> RealmPolicy {
        RealmPolicy {
            history_visibility: "joined".to_owned(),
            join_rule: "invite".to_owned(),
            guest_access: String::new(),
            federate: true,
            encryption_algorithm: String::new(),
            disappearing_policy: None,
            search_policy: None,
        }
    }

    #[test]
    fn disappearing_policy_validates_spec_enums() {
        let mut policy = RealmDisappearingPolicy::default();
        policy.enabled = true;
        policy.max_ttl_ms = 1;
        policy.allowed_triggers = vec!["on_send".to_owned(), "on_first_read".to_owned()];
        policy.validate().unwrap();

        policy.allowed_triggers = vec!["until".to_owned()];
        assert!(policy.validate().unwrap_err().contains("unknown trigger"));
    }

    #[test]
    fn search_policy_rejects_old_or_unknown_fields_by_value() {
        let policy = RealmSearchPolicy {
            enabled_profile_refs: vec!["ck.profile.search.client_index.v1".to_owned()],
            allowed_service_dids: vec!["did:web:search.example".to_owned()],
            data_classes: vec!["encrypted_index".to_owned()],
            index_retention_ms: Some(0),
            revocation_behavior: Some("fail_closed".to_owned()),
        };
        policy.validate().unwrap();

        let mut bad = policy.clone();
        bad.data_classes = vec!["shard_id".to_owned()];
        assert!(bad.validate().unwrap_err().contains("unknown data class"));

        let mut bad = policy;
        bad.revocation_behavior = Some("allow_stale".to_owned());
        assert!(bad.validate().unwrap_err().contains("revocation_behavior"));
    }

    #[test]
    fn realm_policy_validates_nested_policy() {
        let mut policy = base_policy();
        policy.disappearing_policy = Some(RealmDisappearingPolicy {
            enabled: true,
            max_ttl_ms: 0,
            allowed_triggers: vec!["on_send".to_owned()],
            default_grace_ms: None,
            allow_plaintext_realms: Some(false),
        });

        assert!(policy.validate().unwrap_err().contains("max_ttl_ms"));
    }
}
