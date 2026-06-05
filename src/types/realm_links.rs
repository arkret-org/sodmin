//! DTO shapes for the Realm link graph admin surface.

use serde::{Deserialize, Serialize};

// ── Realm link graph (R5.2, Round R1.2 — ck.realm.link projection) ──

/// One outbound / inbound typed link between two Realm boundaries.
/// Backed by `ck.realm.link` reducer_input events. Common `link_kind`
/// values include `governed_by`, `discoverable_from`, `mirror_of`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RealmLinkRow {
    /// Source Realm id (the "from" boundary).
    #[serde(default)]
    pub source_realm_id: String,
    /// Target Realm id (the "to" boundary).
    #[serde(default)]
    pub target_realm_id: String,
    /// Typed link kind. Free-form string at the wire layer; the UI
    /// recognises a small set and renders the rest as `other`.
    #[serde(default)]
    pub link_kind: String,
    /// Human-friendly label for the target Realm, if soland resolves it.
    #[serde(default)]
    pub target_display_name: Option<String>,
    /// HLC / cell timestamp of the last event that established or
    /// refreshed this link.
    #[serde(default)]
    pub updated_at: Option<String>,
    /// Whether this row appears in the inbound list (vs outbound).
    /// Surface-level convenience; the server fills it when the response
    /// covers both directions in a single payload.
    #[serde(default)]
    pub inbound: bool,
}
