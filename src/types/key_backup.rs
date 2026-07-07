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
    /// `completed` / `partial` / `aborted_by_user` / `policy_denied` /
    /// `evidence_insufficient` / `service_defined` per the schema outcome enum.
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
