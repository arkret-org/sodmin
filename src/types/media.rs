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
    pub encrypted_count: u64,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MediaRow {
    #[serde(default)]
    pub media_type: Option<String>,
    #[serde(default)]
    pub filename: Option<String>,
    #[serde(default)]
    pub realm_id: Option<String>,
    #[serde(default)]
    pub encrypted: bool,
    #[serde(default)]
    pub uploaded_by: Option<String>,
    pub size_bytes: u64,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_row_requires_canonical_size_bytes() {
        let legacy = serde_json::json!({"size": 42});
        assert!(serde_json::from_value::<MediaRow>(legacy).is_err());

        let row: MediaRow = serde_json::from_value(serde_json::json!({"size_bytes": 42}))
            .expect("canonical size_bytes field");
        assert_eq!(row.size_bytes, 42);
    }
}
