//! Device admin API — D14 production endpoint.
//!
//! `GET /_soland/admin/devices` is the typed cursor-paginated production
//! query (SDK `AdminDeviceList`); `filter[name_or_id]` is applied
//! server-side.

use arkret_core::models::AdminDeviceList;

use crate::api::client::{NO_BODY, NoBody, api_client, build_url};
use crate::utils::net::error::HttpError;

pub async fn list_devices(
    cursor: Option<&str>,
    limit: u64,
    search: &str,
) -> Result<AdminDeviceList, HttpError> {
    let limit_str = limit.max(1).to_string();
    let mut params: Vec<(&str, &str)> = vec![("limit", limit_str.as_str())];
    if let Some(cursor) = cursor.filter(|cursor| !cursor.is_empty()) {
        params.push(("cursor", cursor));
    }
    let search = search.trim();
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
    let body = serde_json::json!({});
    let _: NoBody = api_client(&url, "POST", Some(&body)).await?;
    Ok(())
}
