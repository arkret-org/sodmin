//! HTTP client for the per-account device admin surface.
//!
//! Endpoints:
//!
//! - `GET /_coauth/admin/accounts/{account_id}/devices`
//! - `POST /_coauth/admin/accounts/{account_id}/devices/{device_id}/revoke`

use serde::Serialize;

use crate::api::client::{api_client, NoBody, NO_BODY};
use crate::types::coauth_devices::CoauthDeviceRow;
use crate::utils::net::error::HttpError;

#[derive(Debug, Clone, serde::Deserialize)]
struct CoauthDevicesEnvelope {
    data: Vec<CoauthDeviceRow>,
}

#[derive(Debug, Serialize)]
struct RevokeDeviceRequestBody<'a> {
    reason: &'a str,
}

pub async fn list_account_devices(account_id: &str) -> Result<Vec<CoauthDeviceRow>, HttpError> {
    let path = format!(
        "/_coauth/admin/accounts/{}/devices",
        urlencoding::encode(account_id)
    );
    let resp: CoauthDevicesEnvelope = api_client(&path, "GET", NO_BODY).await?;
    Ok(resp.data)
}

pub async fn revoke_account_device(
    account_id: &str,
    device_id: &str,
    reason: &str,
) -> Result<(), HttpError> {
    let reason = reason.trim();
    if reason.is_empty() {
        return Err(HttpError::message("device revoke reason is required"));
    }
    let url = format!(
        "/_coauth/admin/accounts/{}/devices/{}/revoke",
        urlencoding::encode(account_id),
        urlencoding::encode(device_id),
    );
    let body = RevokeDeviceRequestBody { reason };
    let _: NoBody = api_client(&url, "POST", Some(&body)).await?;
    Ok(())
}
