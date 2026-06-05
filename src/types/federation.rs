//! DTO shapes for the federation admin surface.

use serde::{Deserialize, Serialize};

// ── Federation types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FederationPeer {
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub trust_level: Option<String>,
    #[serde(default)]
    pub last_successful_txn: Option<String>,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub retry_interval: u64,
    #[serde(default)]
    pub direction: Option<String>,
    #[serde(default)]
    pub connection_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FederationAllowRule {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub domain: String,
    #[serde(default)]
    pub rule_type: Option<String>,
    #[serde(default)]
    pub polarity: Option<String>,
    #[serde(default)]
    pub action: Option<String>,
    #[serde(default)]
    pub allowlist_enforced: Option<bool>,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AddFederationRuleRequest {
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rule_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polarity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowlist_enforced: Option<bool>,
}
