//! DTO shapes for the applet admin surface.

use serde::{Deserialize, Serialize};

// ── Applet types ──

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Applet {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub applet_type: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub endpoint_url: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub registered_at: Option<String>,
    #[serde(default)]
    pub last_transaction_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RegisterAppletRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applet_type: Option<String>,
    pub endpoint_url: String,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
}
