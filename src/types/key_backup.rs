//! DTO shapes for the key-backup admin surface.

use serde::{Deserialize, Serialize};

// ── Key-backup admin types (B-C) ──

/// REC-1 — read-model summary of an accepted recovery policy. Mirrors
/// spec `recovery-policy.schema.json#/$defs/recovery_policy_summary`
/// (the row shape soland's `recovery-policies` / `recovery-policy`
/// reads project: `policy_id` / `principal_id` / `version` /
/// `trust_domain` / `allowed_proof_kinds` / `supersedes` / timestamps +
/// the full `policy` object when disclosable).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecoveryPolicySummary {
    #[serde(default)]
    pub policy_id: String,
    #[serde(default)]
    pub principal_id: String,
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub trust_domain: String,
    #[serde(default)]
    pub allowed_proof_kinds: Vec<String>,
    #[serde(default)]
    pub supersedes: Option<String>,
    #[serde(default)]
    pub issued_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub accepted_at: Option<String>,
    /// Full accepted `ck.schema.recovery_policy.v1` object when the
    /// caller is authorized to receive holder/share details.
    #[serde(default)]
    pub policy: Option<serde_json::Value>,
}

/// `GET /_cokret/self/keys/backups` outcome. Matches the spec
/// `keys_backups_list` schema (`{backups, has_more, next_cursor?}`) rather
/// than the generic `{data, total, next_cursor}` envelope — the latter
/// deserialized `backups` into nothing and rendered a silently-empty list.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KeyBackupListOutcome {
    #[serde(default)]
    pub backups: Vec<KeyBackupSeries>,
    #[serde(default)]
    pub has_more: bool,
    #[serde(default)]
    pub next_cursor: Option<String>,
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

/// REC-1 — recovery receipt history row. Field names follow spec
/// `recovery-receipt.schema.json` (`ck.schema.recovery_receipt.v1`):
/// `receipt_id` / `recovery_session_id` / `policy_id` / `policy_version`
/// / `trust_domain` / `new_device_id` / `outcome` / `completed_at`, plus
/// the acceptance timestamp and the full `receipt` object soland's
/// `recovery-receipts` read projects.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecoveryReceiptSummary {
    #[serde(default)]
    pub receipt_id: String,
    #[serde(default)]
    pub recovery_session_id: String,
    #[serde(default)]
    pub policy_id: String,
    #[serde(default)]
    pub policy_version: u32,
    #[serde(default)]
    pub trust_domain: String,
    #[serde(default)]
    pub new_device_id: String,
    /// `success` / `failure` per the schema outcome enum.
    #[serde(default)]
    pub outcome: String,
    #[serde(default)]
    pub completed_at: Option<String>,
    #[serde(default)]
    pub accepted_at: Option<String>,
    /// Full `ck.schema.recovery_receipt.v1` object.
    #[serde(default)]
    pub receipt: Option<serde_json::Value>,
}
