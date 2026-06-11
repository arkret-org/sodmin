//! DTO shapes for the device admin surface.

use serde::{Deserialize, Serialize};

// ── Device types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Device {
    #[serde(default, alias = "device_id")]
    pub id: String,
    #[serde(default)]
    #[serde(alias = "actor")]
    pub actor_id: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub device_type: Option<String>,
    #[serde(default)]
    pub last_seen_ip: Option<String>,
    #[serde(default)]
    pub last_seen_ts: Option<u64>,
    #[serde(default, alias = "verification_state")]
    pub verification_status: Option<String>,
    #[serde(default)]
    pub is_cross_signed: bool,
}
