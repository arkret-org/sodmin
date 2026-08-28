//! Cross-page cursor pagination value used by the admin SPA.

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
