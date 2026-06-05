//! HTTP client for the soland Realm policy editor
//!
//! - `GET  /_soland/admin/realms/{realm_id}/policy` — current `ck.component.realm.policy.v1` value.
//! - `POST /_soland/admin/realms/{realm_id}/policy` — write a new policy. soland wraps the body
//!   into a cas-register Move.

use crate::api::client::api_client;
use crate::types::realm_policy::{RealmPolicy, UpdateRealmPolicyRequest};
use crate::utils::net::error::HttpError;

pub async fn get_policy(realm_id: &str) -> Result<RealmPolicy, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/policy",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}

pub async fn update_policy(
    realm_id: &str,
    body: &UpdateRealmPolicyRequest,
) -> Result<RealmPolicy, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/policy",
        urlencoding::encode(realm_id)
    );
    let payload = serde_json::to_string(body).unwrap_or_default();
    api_client(&url, "POST", Some(payload)).await
}
