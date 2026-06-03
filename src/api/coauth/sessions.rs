//! coauth admin OAuth2 sessions and personal sessions.

use serde::{Deserialize, Serialize};

use crate::api::client::{api_client, build_url};
use crate::api::openapi_contract::coauth as coauth_paths;
use crate::types::PaginatedResponse;
use crate::utils::net::error::HttpError;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthOAuth2Session {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub human_name: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub finished_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthPersonalSession {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub last_active_at: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
}

pub async fn list_oauth2_sessions(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthOAuth2Session>, HttpError> {
    let url = build_url(
        coauth_paths::OAUTH2_SESSIONS,
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn finish_oauth2_session(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_soland/admin/oauth2-sessions/{}/finish",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

pub async fn list_personal_sessions(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthPersonalSession>, HttpError> {
    let url = build_url(
        coauth_paths::PERSONAL_SESSIONS,
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn create_personal_session(name: &str) -> Result<CoauthPersonalSession, HttpError> {
    let body = serde_json::json!({ "name": name });
    api_client(
        "/_soland/admin/personal-sessions",
        "POST",
        Some(body.to_string()),
    )
    .await
}

pub async fn revoke_personal_session(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_soland/admin/personal-sessions/{}/revoke",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

pub async fn regenerate_personal_session(id: &str) -> Result<CoauthPersonalSession, HttpError> {
    let url = format!(
        "/_soland/admin/personal-sessions/{}/regenerate",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}
