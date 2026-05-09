//! HTTP client for the soland authz capability-grant admin surface
//! (Round 24, B5).
//!
//! Endpoints:
//!
//! - `GET /api/admin/v1/authz/capabilities` — list capability grants
//!   visible to the current admin scope. Optional cursor and filters
//!   (holder / peer / scope) narrow the projection. The page mirrors
//!   coauth's account-list cursor model: server emits `links.next`,
//!   sodmin pushes/pops cursors on the client.
//! - `POST /api/admin/v1/authz/capabilities/{grant_id}/revoke` — admin
//!   revoke. Same 404-tolerant pattern as the other Stream H' admin
//!   actions: when soland hasn't wired the route yet, the page surfaces
//!   a "not yet wired" toast rather than a generic error.

use crate::api::client::{api_client, build_url};
use crate::types::authz::{AuthzCapabilityGrant, AuthzGrantFilter};
use crate::utils::error::HttpError;

/// Cursor-shaped page wrapper for the authz admin surface. Matches the
/// JSON:API-ish envelope soland emits for paginated admin endpoints.
#[derive(Debug, Clone, Default)]
pub struct AuthzGrantPage {
    pub data: Vec<AuthzCapabilityGrant>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct AuthzGrantsEnvelope {
    #[serde(default)]
    data: Vec<AuthzCapabilityGrant>,
    #[serde(default)]
    next_cursor: Option<String>,
    #[serde(default)]
    total: Option<u64>,
}

/// Fetch a page of capability grants. `cursor` is the opaque server-issued
/// cursor; pass `None` for the first page.
pub async fn list_capability_grants(
    cursor: Option<&str>,
    limit: u64,
    filter: &AuthzGrantFilter,
) -> Result<AuthzGrantPage, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![
        ("filter[holder]", filter.holder.trim()),
        ("filter[peer]", filter.peer.trim()),
        ("filter[scope]", filter.scope.trim()),
        ("limit", limit_str.as_str()),
    ];
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", cursor));
    }
    let url = build_url("/api/admin/v1/authz/capabilities", &params)?;
    let resp: AuthzGrantsEnvelope = api_client(&url, "GET", None).await?;
    Ok(AuthzGrantPage {
        data: resp.data,
        next_cursor: resp.next_cursor,
        total: resp.total,
    })
}

/// Revoke a single capability grant. soland is expected to emit an
/// idempotent `cx.cell.authz.capability.revoke.v1` Move; the route is
/// 404-tolerant on the client side.
pub async fn revoke_capability_grant(grant_id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/authz/capabilities/{}/revoke",
        urlencoding::encode(grant_id)
    );
    let _: serde_json::Value = api_client(&url, "POST", None).await?;
    Ok(())
}
