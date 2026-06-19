//! DTO shapes for the policy admin surface.

use serde::{Deserialize, Serialize};

// ── Policy types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Policy {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub policy_type: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub rules: Option<serde_json::Value>,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub guardrails: PolicyGuardrailSummary,
    #[serde(default)]
    pub safety: PolicySafetySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PolicyGuardrailSummary {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub required_scope: Option<String>,
    #[serde(default)]
    pub approval_evidence: Vec<PolicyEvidenceItem>,
    #[serde(default)]
    pub audit_trail: Vec<PolicyAuditEntry>,
    #[serde(default)]
    pub obligations: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PolicyEvidenceItem {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub reference: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issued_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PolicyAuditEntry {
    #[serde(default)]
    pub action: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PolicySafetySummary {
    #[serde(default)]
    pub read_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_only_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pin_summary: Option<PinPolicySummary>,
    #[serde(default)]
    pub redacted_private_categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct PinPolicySummary {
    #[serde(default)]
    pub standard_surface_available: bool,
    #[serde(default)]
    pub actions: Vec<String>,
    #[serde(default)]
    pub pin_scopes: Vec<String>,
    #[serde(default)]
    pub quota_limits: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note_plaintext_policy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreatePolicyRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<serde_json::Value>,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub priority: i32,
}
