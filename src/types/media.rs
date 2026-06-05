//! DTO shapes for the media admin surface.

use serde::{Deserialize, Serialize};

// ── Media / Blob types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaStatistics {
    #[serde(default)]
    pub total_blobs: u64,
    #[serde(default)]
    pub total_size: u64,
    #[serde(default)]
    pub quarantined_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActorMediaStatistics {
    #[serde(default)]
    pub actor_id: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub blob_count: u64,
    #[serde(default)]
    pub total_size: u64,
}
