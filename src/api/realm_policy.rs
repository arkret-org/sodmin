//! HTTP client for the soland Realm policy editor
//!
//! Realm policy writes are not an admin DB wrapper. The read path is a
//! best-effort projection from the Realm admin snapshot; mutation waits
//! for the B6 event-backed governance decision.

use crate::api::client::api_client;
use crate::types::realm_policy::{RealmPolicy, UpdateRealmPolicyRequest};
use crate::utils::net::error::HttpError;

pub async fn get_policy(realm_id: &str) -> Result<RealmPolicy, HttpError> {
    let url = format!("/_soland/admin/realms/{}", urlencoding::encode(realm_id));
    let value: serde_json::Value = api_client(&url, "GET", None).await?;
    let Some(policy) = value.get("policy").or_else(|| value.get("realm_policy")) else {
        return Err(HttpError::message(
            "realm policy projection is not wired on the admin realm snapshot",
        ));
    };
    serde_json::from_value(policy.clone()).map_err(|error| {
        HttpError::message(format!("realm policy projection parse error: {error}"))
    })
}

pub async fn update_policy(
    realm_id: &str,
    body: &UpdateRealmPolicyRequest,
) -> Result<RealmPolicy, HttpError> {
    let _ = (realm_id, body);
    Err(HttpError::message(
        "realm policy write endpoint is not wired",
    ))
}
