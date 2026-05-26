use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

/// Cursor-paginated device list. `cursor` is the opaque token returned
/// by the previous page (or `None` for the first page). `search` is a
/// best-effort `name_or_id` filter — backends that don't yet plumb it
/// just return the unfiltered page and the page does its own
/// client-side filter (`utils::search::matches_name_or_id`).
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
    let url = build_url("/api/admin/v1/devices", &params)?;
    api_client(&url, "GET", None).await
}

pub async fn delete_device(id: &str) -> Result<(), HttpError> {
    let url = format!("/api/admin/v1/devices/{}", urlencoding::encode(id));
    api_client(&url, "DELETE", None).await
}

/// Bulk-revoke helper — same wire shape as `delete_device` but hits the
/// soland `/revoke` action which signs out the live session and drops
/// device keys in addition to deleting the row. Used by the device list
/// "Bulk Revoke" toolbar.
pub async fn revoke_device(id: &str) -> Result<(), HttpError> {
    let url = format!("/api/admin/v1/devices/{}/revoke", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}
