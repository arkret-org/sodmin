//! HTTP client for the per-account device admin surface
//!
//! Endpoints:
//!
//! - `GET /_soland/admin/accounts/{account_id}/devices` — coauth list of devices registered to a
//!   single account. Cursor-paginated.
//! - `POST /_soland/admin/accounts/{account_id}/devices/{device_id}/revoke` — coauth revoke. The
//!   cascade revoke of session grants on the soland side is wired in coauth round 23; the admin UI
//!   just calls the coauth route. 404-tolerant on the client side.

use crate::api::client::{api_client, build_url};
use crate::types::coauth_devices::CoauthDeviceRow;
use crate::utils::error::HttpError;

#[derive(Debug, Clone, Default)]
pub struct CoauthDevicePage {
    pub data: Vec<CoauthDeviceRow>,
    pub next_cursor: Option<String>,
    pub total: Option<u64>,
}

#[derive(Debug, Clone, serde::Deserialize, Default)]
struct CoauthDevicesEnvelope {
    #[serde(default)]
    data: Vec<CoauthDeviceRow>,
    #[serde(default)]
    next_cursor: Option<String>,
    #[serde(default)]
    total: Option<u64>,
}

pub async fn list_account_devices(
    account_id: &str,
    cursor: Option<&str>,
    limit: u64,
) -> Result<CoauthDevicePage, HttpError> {
    let limit_str = limit.max(1).to_string();
    let path = format!(
        "/_soland/admin/accounts/{}/devices",
        urlencoding::encode(account_id)
    );
    let mut params: Vec<(&str, &str)> = vec![("limit", limit_str.as_str())];
    if let Some(cursor) = cursor.filter(|c| !c.is_empty()) {
        params.push(("cursor", cursor));
    }
    let url = build_url(&path, &params)?;
    let resp: CoauthDevicesEnvelope = api_client(&url, "GET", None).await?;
    Ok(CoauthDevicePage {
        data: resp.data,
        next_cursor: resp.next_cursor,
        total: resp.total,
    })
}

pub async fn revoke_account_device(account_id: &str, device_id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_soland/admin/accounts/{}/devices/{}/revoke",
        urlencoding::encode(account_id),
        urlencoding::encode(device_id),
    );
    let _: serde_json::Value = api_client(&url, "POST", None).await?;
    Ok(())
}
