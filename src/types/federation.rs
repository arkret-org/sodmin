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
