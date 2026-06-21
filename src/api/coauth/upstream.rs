//! coauth admin upstream OAuth providers and account links.

use serde::{Deserialize, Serialize};

use crate::api::client::{api_client, build_url};
use crate::types::PaginatedResponse;
use crate::utils::net::error::HttpError;

const UPSTREAM_OAUTH_PROVIDERS_PATH: &str = "/_coauth/admin/upstream-oauth-providers";
const UPSTREAM_OAUTH_LINKS_PATH: &str = "/_coauth/admin/upstream-oauth-links";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthUpstreamProvider {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub issuer: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthUpstreamLink {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub provider_id: Option<String>,
    #[serde(default)]
    pub subject: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

pub async fn list_upstream_providers(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthUpstreamProvider>, HttpError> {
    let url = build_url(
        UPSTREAM_OAUTH_PROVIDERS_PATH,
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn create_upstream_provider(
    provider: &serde_json::Value,
) -> Result<CoauthUpstreamProvider, HttpError> {
    api_client(
        "/_coauth/admin/upstream-oauth-providers",
        "POST",
        Some(provider.to_string()),
    )
    .await
}

pub async fn delete_upstream_provider(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_coauth/admin/upstream-oauth-providers/{}",
        urlencoding::encode(id)
    );
    api_client(&url, "DELETE", None).await
}

pub async fn toggle_upstream_provider(id: &str, enable: bool) -> Result<(), HttpError> {
    let action = if enable { "enable" } else { "disable" };
    let url = format!(
        "/_coauth/admin/upstream-oauth-providers/{}/{}",
        urlencoding::encode(id),
        action
    );
    api_client(&url, "POST", None).await
}

pub async fn list_upstream_links(
    page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthUpstreamLink>, HttpError> {
    let url = build_url(
        UPSTREAM_OAUTH_LINKS_PATH,
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn delete_upstream_link(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_coauth/admin/upstream-oauth-links/{}",
        urlencoding::encode(id)
    );
    api_client(&url, "DELETE", None).await
}
