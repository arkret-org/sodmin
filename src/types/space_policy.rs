//! Admin DTOs for the soland Space Policy editor.
//!
//! Mirrors the `ck.component.space.policy.v1` component body. Submit
//! constructs a cas-register Move via
//! `POST /_soland/admin/spaces/{id}/policy` with a typed body the
//! backend wraps into a Move + signature.
//!
//! These types previously lived in `coauth-admin-types::space_policy_admin`;
//! that module was removed when coauth narrowed the shared crate, so sodmin
//! now owns the read/write projection locally.

use serde::{Deserialize, Serialize};

/// Components inside a Space policy. Each lives at a known component
/// state-key; the editor exposes them in a flat form so the operator
/// edits the whole policy as one transaction.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpacePolicy {
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
    /// Federate the Space at all. `false` = same-server only.
    #[serde(default = "default_federate")]
    pub federate: bool,
    /// Encryption algorithm — empty string = no E2EE.
    #[serde(default)]
    pub encryption_algorithm: String,
}

fn default_federate() -> bool {
    true
}

impl SpacePolicy {
    /// Validate that the policy is internally consistent. Returns the
    /// first invariant violation as a human-readable string. Used by the
    /// editor before posting the cas-register Move.
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
        Ok(())
    }
}

/// Body `POSTed` to `/_soland/admin/spaces/{id}/policy`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateSpacePolicyRequest {
    pub policy: SpacePolicy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}
