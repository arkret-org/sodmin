//! Cross-page pagination value types shared by the admin SPA.
//!
//! These are deliberately kept separate from the JSON:API envelope
//! (`coauth_admin_types::PaginatedResponse` — `{data, meta, links}`) and
//! from the soland flat list envelope ([`crate::types::api::ListResponse`]
//! — `{data, total?, next_cursor}`). The wire shapes differ field-for-field,
//! so they are NOT interchangeable and must not be merged with JSON:API
//! pagination.

use serde::Deserialize;

/// Flat `{data, total}` page envelope used by the coauth admin
/// page/per-page list endpoints (audit-feed, sessions, providers, …).
///
/// Distinct from the JSON:API envelope (which nests each item in a
/// `SingleResource` and carries `meta`/`links`) and from
/// [`crate::types::api::ListResponse`] (which additionally carries an
/// opaque `next_cursor`).
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub total: u64,
}

/// Cursor-shaped pagination result. `next_cursor` is `Some` when the
/// server links another page. It is an opaque base64url string produced
/// by the backend and fed back unchanged on the next request.
///
/// This is a SPA-local aggregate (no `Serialize`/`Deserialize`) assembled
/// from a JSON:API paginated envelope; it never appears on the wire.
#[derive(Debug, Clone, Default)]
pub struct CursorPage<T> {
    pub data: Vec<T>,
    pub next_cursor: Option<String>,
    /// Total when the server populated `meta.count` (best-effort).
    pub total: Option<u64>,
}
