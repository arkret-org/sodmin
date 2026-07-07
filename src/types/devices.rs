//! DTO shapes for the device admin surface.

use serde::{Deserialize, Serialize};

// ── Device types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Device {
    #[serde(default, alias = "device_id")]
    pub id: String,
    #[serde(default, alias = "actor")]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub verification_status: Option<String>,
    #[serde(default, alias = "verification", alias = "verification_state")]
    pub verification_state: Option<String>,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub revoked_at: Option<String>,
}

impl Device {
    pub fn verification_label(&self) -> Option<&str> {
        self.verification_state
            .as_deref()
            .or(self.verification_status.as_deref())
    }
}
