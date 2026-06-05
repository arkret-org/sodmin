//! DTO shapes for the invite-token admin surface.

use serde::{Deserialize, Serialize};

// ── Invite token types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InviteToken {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub token: String,
    #[serde(default)]
    pub uses_allowed: Option<u64>,
    #[serde(default)]
    pub uses_completed: u64,
    #[serde(default)]
    pub uses_pending: u64,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub realm_id: Option<String>,
    // Audit pair: `created_by` precedes `created_at`, matching the
    // project-wide (and canonical) ordering convention.
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateInviteTokenRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uses_allowed: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realm_id: Option<String>,
}
