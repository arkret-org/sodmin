//! DTO shapes for the actor admin surface.

use serde::{Deserialize, Serialize};

// ── Actor types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Actor {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub account_id: Option<String>,
    #[serde(default)]
    pub did: String,
    #[serde(default)]
    pub handle: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub avatar_url: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default)]
    pub is_suspended: bool,
    #[serde(default)]
    pub is_deactivated: bool,
    /// Round 4 — `true` when the local 7-domain fanout completed but at
    /// least one federated peer has NOT yet confirmed the deactivation.
    /// The admin UI MUST NOT silently treat this principal as fully
    /// deactivated — surface the incomplete state explicitly.
    #[serde(default)]
    pub deactivation_federation_incomplete: bool,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_active_at: Option<String>,
    #[serde(default)]
    pub device_count: u64,
    #[serde(default)]
    pub realm_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateActorRequest {
    #[serde(default)]
    pub handle: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub is_admin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateActorRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_admin: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_suspended: Option<bool>,
}
