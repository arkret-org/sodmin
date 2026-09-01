//! coauth admin upstream OAuth providers and account links.

use serde::{Deserialize, Serialize};

use super::pagination::{get_jsonapi_cursor_page, map_single_resource};
use crate::api::client::{NO_BODY, NoBody, api_client};
use crate::types::CursorPage;
use crate::utils::net::error::HttpError;

const UPSTREAM_OAUTH_PROVIDERS_PATH: &str = "/_coauth/admin/upstream-oauth-providers";
const UPSTREAM_OAUTH_LINKS_PATH: &str = "/_coauth/admin/upstream-oauth-links";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthUpstreamProvider {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub oidc_issuer_uri: Option<String>,
    #[serde(default)]
    pub is_enabled: bool,
    #[serde(default)]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateUpstreamProviderRequest {
    pub oidc_issuer_uri: String,
    pub client_id: String,
}

#[derive(Debug, Clone, Serialize)]
struct UpstreamProviderRequestBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    oidc_issuer_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    human_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    brand_name: Option<String>,
    scope: String,
    token_endpoint_auth_method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    token_endpoint_signing_alg: Option<String>,
    id_token_signed_response_alg: String,
    fetch_userinfo: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    userinfo_signed_response_alg: Option<String>,
    client_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_secret: Option<String>,
    claims_imports: serde_json::Value,
    discovery_mode: String,
    pkce_mode: String,
    on_backchannel_logout: String,
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
    after: Option<&str>,
    per_page: u64,
) -> Result<CursorPage<CoauthUpstreamProvider>, HttpError> {
    get_jsonapi_cursor_page(
        UPSTREAM_OAUTH_PROVIDERS_PATH,
        after,
        per_page,
        map_upstream_provider,
    )
    .await
}

pub async fn create_upstream_provider(
    provider: &CreateUpstreamProviderRequest,
) -> Result<CoauthUpstreamProvider, HttpError> {
    let body = UpstreamProviderRequestBody {
        oidc_issuer_uri: Some(provider.oidc_issuer_uri.clone())
            .filter(|value| !value.trim().is_empty()),
        human_name: None,
        brand_name: None,
        scope: "openid".to_owned(),
        token_endpoint_auth_method: "none".to_owned(),
        token_endpoint_signing_alg: None,
        id_token_signed_response_alg: "RS256".to_owned(),
        fetch_userinfo: false,
        userinfo_signed_response_alg: None,
        client_id: provider.client_id.clone(),
        client_secret: None,
        claims_imports: serde_json::json!({}),
        discovery_mode: "oidc".to_owned(),
        pkce_mode: "auto".to_owned(),
        on_backchannel_logout: "do_nothing".to_owned(),
    };
    let resp: coauth_admin_types::SingleOutcome<coauth_admin_types::UpstreamOAuthProvider> =
        api_client(UPSTREAM_OAUTH_PROVIDERS_PATH, "POST", Some(&body)).await?;
    Ok(map_single_resource(resp, map_upstream_provider))
}

pub async fn delete_upstream_provider(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_coauth/admin/upstream-oauth-providers/{}",
        urlencoding::encode(id)
    );
    let _: NoBody = api_client(&url, "DELETE", NO_BODY).await?;
    Ok(())
}

pub async fn toggle_upstream_provider(id: &str, enable: bool) -> Result<(), HttpError> {
    let action = if enable { "enable" } else { "disable" };
    let url = format!(
        "/_coauth/admin/upstream-oauth-providers/{}/{}",
        urlencoding::encode(id),
        action
    );
    let _: NoBody = api_client(&url, "POST", NO_BODY).await?;
    Ok(())
}

pub async fn list_upstream_links(
    after: Option<&str>,
    per_page: u64,
) -> Result<CursorPage<CoauthUpstreamLink>, HttpError> {
    get_jsonapi_cursor_page(
        UPSTREAM_OAUTH_LINKS_PATH,
        after,
        per_page,
        map_upstream_link,
    )
    .await
}

pub async fn delete_upstream_link(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_coauth/admin/upstream-oauth-links/{}",
        urlencoding::encode(id)
    );
    let _: NoBody = api_client(&url, "DELETE", NO_BODY).await?;
    Ok(())
}

fn map_upstream_provider(
    resource: coauth_admin_types::SingleResource<coauth_admin_types::UpstreamOAuthProvider>,
) -> CoauthUpstreamProvider {
    let attrs = resource.attributes;
    CoauthUpstreamProvider {
        id: resource.id,
        oidc_issuer_uri: attrs.oidc_issuer_uri,
        is_enabled: attrs.disabled_at.is_none(),
        created_at: Some(attrs.created_at.to_rfc3339()),
    }
}

fn map_upstream_link(
    resource: coauth_admin_types::SingleResource<coauth_admin_types::UpstreamOAuthLink>,
) -> CoauthUpstreamLink {
    let attrs = resource.attributes;
    CoauthUpstreamLink {
        id: resource.id,
        user_id: attrs.user_id,
        provider_id: Some(attrs.provider_id).filter(|value| !value.is_empty()),
        subject: Some(attrs.subject).filter(|value| !value.is_empty()),
        created_at: Some(attrs.created_at.to_rfc3339()),
    }
}
