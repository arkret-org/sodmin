//! HTTP client for the Notary / Seal / Bottom admin endpoints exposed by
//! soland.

use crate::api::client::{NO_BODY, api_client, build_url};
use crate::types::seal::{AdminNotaryValue, BottomEntry, SealDagSnapshot};
use crate::utils::net::error::HttpError;

pub async fn get_notary_value(realm_id: &str) -> Result<AdminNotaryValue, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/notary",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", NO_BODY).await
}

pub async fn list_bottom_entries_global() -> Result<Vec<BottomEntry>, HttpError> {
    let url = build_url("/_soland/admin/bottom", &[])?;
    api_client(&url, "GET", NO_BODY).await
}

pub async fn get_seal_dag(realm_id: &str) -> Result<SealDagSnapshot, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/seal-dag",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", NO_BODY).await
}
