//! HTTP client for the soland Handle admin surface (T6.2 §2).
//!
//! Endpoints (all 404-tolerant — older servers may not yet expose them):
//!
//! - `GET /_soland/admin/handles` — paginated list of `cx.handle.*` cells visible to the current
//!   admin scope.
//! - `GET /_soland/admin/handles/{id}` — single handle row.
//! - `GET /_soland/admin/handles/{id}/audit` — handle audit trail from the T3.2 audit table.
//! - `POST /_soland/admin/handles/{id}/revoke` — publish a revoke Move.
//! - `POST /_soland/admin/handles/{id}/reassign` — force a re-bind to a new subject DID.

use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn list_handles(
    page: u64,
    per_page: u64,
    search: &str,
) -> Result<ListResponse<HandleRecord>, HttpError> {
    let url = build_url(
        "/_soland/admin/handles",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
            ("search", search),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn get_handle(id: &str) -> Result<HandleRecord, HttpError> {
    let url = format!("/_soland/admin/handles/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

pub async fn get_handle_audit(id: &str) -> Result<ListResponse<HandleAuditEvent>, HttpError> {
    let url = format!("/_soland/admin/handles/{}/audit", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

pub async fn revoke_handle(id: &str) -> Result<(), HttpError> {
    let url = format!("/_soland/admin/handles/{}/revoke", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

pub async fn reassign_handle(
    id: &str,
    req: &HandleReassignRequest,
) -> Result<HandleRecord, HttpError> {
    let url = format!("/_soland/admin/handles/{}/reassign", urlencoding::encode(id));
    api_client(
        &url,
        "POST",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}
