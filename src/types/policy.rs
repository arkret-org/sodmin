//! DTO shapes for the policy admin surface.

use serde::Serialize;

// ── Policy types ──

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(transparent)]
pub struct PolicyRuleSet(serde_json::Value);

impl Default for PolicyRuleSet {
    fn default() -> Self {
        Self(serde_json::json!({}))
    }
}

impl PolicyRuleSet {
    pub fn as_value(&self) -> &serde_json::Value {
        &self.0
    }

    pub fn into_value(self) -> serde_json::Value {
        self.0
    }
}

impl From<serde_json::Value> for PolicyRuleSet {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(transparent)]
pub struct PolicyObligation(serde_json::Value);

impl From<serde_json::Value> for PolicyObligation {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AdminPolicy {
    pub id: String,
    pub name: String,
    pub policy_kind: Option<String>,
    pub description: Option<String>,
    pub scope: Option<String>,
    pub subject_ref: Option<String>,
    pub rules: Option<PolicyRuleSet>,
    pub is_enabled: bool,
    pub priority: i32,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub guardrails: PolicyGuardrailSummary,
    pub safety: PolicySafetySummary,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PolicyGuardrailSummary {
    /// Typed top-level obligations owned by `AdminPolicyPayload`.
    ///
    /// The payload's `resource` member is deliberately open operator JSON;
    /// no approval-evidence or audit-trail shape is inferred from it.
    pub obligations: Vec<PolicyObligation>,
}

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct PolicySafetySummary {
    pub read_only: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_only_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct CreatePolicyRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<PolicyRuleSet>,
    pub is_enabled: bool,
    pub priority: i32,
}

pub type AdminPolicyListOutcome = crate::types::api::ListResponse<AdminPolicy>;
