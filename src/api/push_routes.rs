//! HTTP client for the soland push-route admin surface (T6.2 §4).
//!
//! Walks the `cx.device.push_route` cell tree, grouped by principal.
//! `push_target_id` is sensitive — the admin UI must keep it collapsed
//! by default and surface it behind an explicit "reveal" affordance.

use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn list_push_routes(
    page: u64,
    per_page: u64,
    principal_id: &str,
) -> Result<ListResponse<PushRouteRow>, HttpError> {
    let url = build_url(
        "/api/admin/v1/push-routes",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
            ("principal_id", principal_id),
        ],
    )?;
    api_client(&url, "GET", None).await
}
