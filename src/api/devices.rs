use crate::api::client::{NO_BODY, api_client, build_url};
use crate::types::*;
use crate::utils::net::error::HttpError;

/// Cursor-paginated device list. `cursor` is the opaque token returned
/// by the previous page (or `None` for the first page). `search` is a
/// best-effort `name_or_id` filter — backends that don't yet plumb it
/// just return the unfiltered page and the page does its own
/// client-side filter (`utils::fmt::search::matches_name_or_id`).
pub async fn list_devices(
    cursor: Option<&str>,
    limit: u64,
    search: &str,
) -> Result<ListResponse<Device>, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![("limit", limit_str.as_str())];
    if let Some(c) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", c));
    }
    if !search.is_empty() {
        params.push(("filter[name_or_id]", search));
    }
    let url = build_url("/_soland/admin/devices", &params)?;
    api_client(&url, "GET", NO_BODY).await
}

/// Revoke helper. Revocation is terminal and audit-retaining; sodmin does
/// not issue physical DELETEs for devices.
pub async fn revoke_device(id: &str) -> Result<(), HttpError> {
    let url = format!("/_soland/admin/devices/{}/revoke", urlencoding::encode(id));
    api_client(&url, "POST", NO_BODY).await
}
