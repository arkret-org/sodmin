//! coauth admin user-registration tokens.

use serde::{Deserialize, Serialize};

use super::pagination::{get_jsonapi_cursor_page, map_single_resource};
use crate::api::client::{NoBody, api_client};
use crate::types::CursorPage;
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

#[derive(Debug, Clone, Serialize)]
struct CreateRegistrationTokenBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    usage_limit: Option<u32>,
}

pub async fn list_registration_tokens(
    after: Option<&str>,
    per_page: u64,
) -> Result<CursorPage<CoauthRegistrationToken>, HttpError> {
    get_jsonapi_cursor_page(
        USER_REGISTRATION_TOKENS_PATH,
        after,
        per_page,
        map_registration_token,
    )
    .await
}

pub async fn create_registration_token(
    uses_allowed: Option<u64>,
) -> Result<CoauthRegistrationToken, HttpError> {
    let body = CreateRegistrationTokenBody {
        usage_limit: uses_allowed.map(|value| value.min(u32::MAX as u64) as u32),
    };
    let resp: coauth_admin_types::SingleOutcome<coauth_admin_types::UserRegistrationToken> =
        api_client(USER_REGISTRATION_TOKENS_PATH, "POST", Some(&body)).await?;
    Ok(map_single_resource(resp, map_registration_token))
}

pub async fn revoke_registration_token(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_coauth/admin/user-registration-tokens/{}/revoke",
        urlencoding::encode(id)
    );
    let _: NoBody = api_client(&url, "POST", None::<()>).await?;
    Ok(())
}

fn map_registration_token(
    resource: coauth_admin_types::SingleResource<coauth_admin_types::UserRegistrationToken>,
) -> CoauthRegistrationToken {
    let attrs = resource.attributes;
    let uses_allowed = attrs.usage_limit.map(u64::from);
    let uses_completed = u64::from(attrs.times_used);
    let uses_pending = uses_allowed
        .map(|limit| limit.saturating_sub(uses_completed))
        .unwrap_or_default();
    CoauthRegistrationToken {
        id: resource.id,
        token: Some(attrs.token).filter(|token| !token.is_empty()),
        uses_allowed,
        uses_completed,
        uses_pending,
        expires_at: attrs.expires_at.map(|value| value.to_rfc3339()),
        created_at: Some(attrs.created_at.to_rfc3339()),
        is_revoked: attrs.revoked_at.is_some(),
    }
}
