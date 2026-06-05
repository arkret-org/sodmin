//! HTTP client for the soland push-route admin surface (T6.2 §4).
//!
//! Walks the `ck.device.push_route` cell tree, grouped by principal.
//! `push_target_id` is sensitive — the admin UI must keep it collapsed
//! by default and surface it behind an explicit "reveal" affordance.

use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::net::error::HttpError;

pub async fn list_push_routes(
    cursor: Option<&str>,
    limit: u64,
    principal_id: &str,
) -> Result<ListResponse<PushRouteRow>, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![
        ("limit", limit_str.as_str()),
        ("principal_id", principal_id.trim()),
    ];
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", cursor));
    }
    let url = build_url("/_soland/admin/push-routes", &params)?;
    api_client(&url, "GET", None).await
}
