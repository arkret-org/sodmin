//! HTTP client for the soland Space Policy editor (Round 25, D4).
//!
//! - `GET  /api/admin/v1/spaces/{id}/policy` — current `cx.component.space.policy.v1` value.
//! - `POST /api/admin/v1/spaces/{id}/policy` — write a new policy. soland
//!   wraps the body into a cas-register Move.

use coauth_admin_types::space_policy_admin::{SpacePolicy, UpdateSpacePolicyRequest};

use crate::api::client::api_client;
use crate::utils::error::HttpError;

pub async fn get_policy(space_id: &str) -> Result<SpacePolicy, HttpError> {
    let url = format!(
        "/api/admin/v1/spaces/{}/policy",
        urlencoding::encode(space_id)
    );
    api_client(&url, "GET", None).await
}

pub async fn update_policy(
    space_id: &str,
    body: &UpdateSpacePolicyRequest,
) -> Result<SpacePolicy, HttpError> {
    let url = format!(
        "/api/admin/v1/spaces/{}/policy",
        urlencoding::encode(space_id)
    );
    let payload = serde_json::to_string(body).unwrap_or_default();
    api_client(&url, "POST", Some(payload)).await
}
