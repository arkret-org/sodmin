//! DTO shapes for the Realm admin surface.

use arkret_identifiers::BlobRef;
use arkret_wire::{Discoverability, HistoryVisibility, JoinRule};
use serde::{Deserialize, Serialize};

// ── Realm types (security boundary) ──
//
// The object that carries encryption / join-rule / history-visibility /
// realm-class boundary fields is a Realm. Space containers are represented
// separately by `spaces::SpaceRow`.

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AdminRealm {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub discoverability: Option<Discoverability>,
    #[serde(default)]
    pub created_by: Option<String>,
    #[serde(default)]
    pub member_count: u64,
    #[serde(default)]
    pub is_encrypted: bool,
    #[serde(default)]
    pub is_blocked: bool,
    #[serde(default)]
    pub topic: Option<String>,
    #[serde(default)]
    pub avatar_blob_ref: Option<BlobRef>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default, rename = "default_join_rule")]
    pub join_rule: Option<JoinRule>,
    #[serde(default)]
    pub history_visibility: Option<HistoryVisibility>,
    /// AKP-0007 (P3A.6) — `principal_control` vs `collaboration`.
    #[serde(default)]
    pub realm_class: Option<String>,
}

impl AdminRealm {
    pub fn discoverability_label(&self) -> Option<String> {
        self.discoverability.as_ref().map(wire_label)
    }

    pub fn join_rule_label(&self) -> Option<String> {
        self.join_rule.as_ref().map(wire_label)
    }

    pub fn type_label(&self) -> String {
        self.realm_class
            .as_deref()
            .unwrap_or("collaboration")
            .to_owned()
    }
}

fn wire_label<T>(value: &T) -> String
where
    T: Serialize,
{
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "-".to_owned())
}
