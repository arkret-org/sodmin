//! coauth admin user-registration tokens.

use serde::{Deserialize, Serialize};

use crate::api::client::{NO_BODY, api_client, build_url};
use crate::types::PaginatedResponse;
use crate::utils::net::error::HttpError;

const USER_REGISTRATION_TOKENS_PATH: &str = "/_coauth/admin/user-registration-tokens";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthRegistrationToken {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub uses_allowed: Option<u64>,
    #[serde(default)]
    pub uses_completed: u64,
    #[serde(default)]
    pub uses_pending: u64,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub is_revoked: bool,
}

pub async fn list_registration_tokens(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthRegistrationToken>, HttpError> {
    let url = build_url(
        USER_REGISTRATION_TOKENS_PATH,
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", NO_BODY).await
}

pub async fn create_registration_token(
    uses_allowed: Option<u64>,
) -> Result<CoauthRegistrationToken, HttpError> {
    let body = serde_json::json!({ "uses_allowed": uses_allowed });
    api_client(
        "/_coauth/admin/user-registration-tokens",
        "POST",
        Some(&body),
    )
    .await
}

pub async fn revoke_registration_token(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_coauth/admin/user-registration-tokens/{}/revoke",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", NO_BODY).await
}
