//! HTTP client for the soland Realm `media_service` admin surface.
//!
//! The cell lives at `ak.component.realm.media_service.v1` and is
//! projected per Realm. This surface is **read-only** in sodmin:
//! media_service writes are a general-management action that strands
//! through events / inkson, not the operations console. sodmin only
//! renders the effective `foci[]` set plus the service DID.
//!
//! The admin endpoint is `/_soland/admin/realms/{id}/media-service`
//! (GET only).

use crate::api::client::{NO_BODY, api_client};
use crate::types::*;
use crate::utils::net::error::HttpError;

pub async fn get_realm_media_service(realm_id: &str) -> Result<AdminRealmMediaService, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/media-service",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", NO_BODY).await
}
