//! DTO shapes for the federation admin surface.
//!
//! `GET /_soland/admin/federation` returns a cursor-paginated stream of
//! **federation operations** (not peer-health rows). The wire shape is
//! produced by soland's `admin_federation_items`
//! (`soland/crates/server/src/routing/admin/collection.rs`). Each item is a
//! single federated operation projected from the persistence layer.

use serde::{Deserialize, Serialize};

// ── Federation operation row ──

/// One federated operation as projected by soland's admin federation
/// endpoint. Strongly typed against the server's `json!` shape so a field
/// rename on the server breaks compilation here rather than silently
/// returning empty columns.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FederationOperation {
    /// Stable identifier for the operation; also the detail-page key.
    #[serde(default)]
    pub operation_id: String,
    /// Realm the operation belongs to.
    #[serde(default)]
    pub realm_id: Option<String>,
    /// High-level operation type (e.g. `seal`, `delivery`).
    #[serde(default)]
    pub operation_type: Option<String>,
    /// Canonical event kind string.
    #[serde(default)]
    pub canonical_kind: Option<String>,
    /// Discussion strand the operation projects into, when applicable.
    #[serde(default)]
    pub strand_id: Option<String>,
    /// Discussion track, when applicable.
    #[serde(default)]
    pub track: Option<String>,
    /// Operation digest (absent when digest computation failed server-side).
    #[serde(default)]
    pub digest: Option<String>,
    /// Server-side creation timestamp (RFC 3339).
    #[serde(default)]
    pub created_at: Option<String>,
}
