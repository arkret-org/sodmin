//! DTO shapes for the key-backup admin surface.

use serde::{Deserialize, Serialize};

// ── Key-backup admin types (B-C) ──

/// B-C — recovery policy lifecycle. Mirrors
/// `ck.schema.recovery_policy.v1#lifecycle`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecoveryPolicy {
    #[serde(default)]
    pub policy_id: String,
    /// `pending` / `active` / `retired`.
    #[serde(default)]
    pub lifecycle: String,
    #[serde(default)]
    pub kdf_profile: Option<String>,
    #[serde(default)]
    pub epoch_hash: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyBackupSeries {
    #[serde(default)]
    pub series_id: String,
    #[serde(default)]
    pub backup_class: Option<String>,
    /// Highest `series_seq` observed for this series.
    #[serde(default)]
    pub frontier_seq: u64,
    #[serde(default)]
    pub frontier_ref: Option<String>,
    /// Spec rename (head 37ce729): `series_sequence` → `series_seq`.
    #[serde(default)]
    pub series_seq: u64,
    /// Three-class 409 reason counters per §12.1.
    #[serde(default)]
    pub series_chain_broken_count: u64,
    #[serde(default)]
    pub series_seq_not_monotonic_count: u64,
    #[serde(default)]
    pub series_predecessor_not_found_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecoveryReceipt {
    #[serde(default)]
    pub receipt_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub policy_id: Option<String>,
    #[serde(default)]
    pub issued_at: Option<String>,
    #[serde(default)]
    pub verified: bool,
}
