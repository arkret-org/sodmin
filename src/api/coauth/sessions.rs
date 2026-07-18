//! coauth admin OAuth2 sessions and personal sessions.

use serde::{Deserialize, Serialize};

use super::pagination::{get_jsonapi_first_page, map_single_resource};
use crate::api::client::{NO_BODY, api_client};
use crate::types::PaginatedResponse;
use crate::utils::net::error::HttpError;

const OAUTH2_SESSIONS_PATH: &str = "/_coauth/admin/oauth-sessions";
const PERSONAL_SESSIONS_PATH: &str = "/_coauth/admin/personal-sessions";

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
pub struct CoauthPersonalSessionRow {
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
}

pub type CoauthPersonalSession = CoauthPersonalSessionRow;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[non_exhaustive]
pub struct CoauthPersonalSessionOneShot {
    #[serde(flatten)]
    pub session: CoauthPersonalSessionRow,
    #[serde(default)]
    pub access_token: Option<String>,
}

pub async fn list_oauth2_sessions(
    _page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthOAuth2Session>, HttpError> {
    get_jsonapi_first_page(OAUTH2_SESSIONS_PATH, per_page, map_oauth2_session).await
}

pub async fn finish_oauth2_session(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_coauth/admin/oauth-sessions/{}/finish",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", NO_BODY).await
}

pub async fn list_personal_sessions(
    _page: u64,
    per_page: u64,
) -> Result<PaginatedResponse<CoauthPersonalSession>, HttpError> {
    get_jsonapi_first_page(PERSONAL_SESSIONS_PATH, per_page, map_personal_session).await
}

pub async fn create_personal_session(
    name: &str,
) -> Result<CoauthPersonalSessionOneShot, HttpError> {
    let body = serde_json::json!({ "name": name });
    let resp: coauth_admin_types::SingleOutcome<coauth_admin_types::PersonalSession> =
        api_client(PERSONAL_SESSIONS_PATH, "POST", Some(&body)).await?;
    Ok(map_single_resource(resp, map_personal_session_oneshot))
}

pub async fn revoke_personal_session(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_coauth/admin/personal-sessions/{}/revoke",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", NO_BODY).await
}

pub async fn regenerate_personal_session(
    id: &str,
) -> Result<CoauthPersonalSessionOneShot, HttpError> {
    let url = format!(
        "/_coauth/admin/personal-sessions/{}/regenerate",
        urlencoding::encode(id)
    );
    let resp: coauth_admin_types::SingleOutcome<coauth_admin_types::PersonalSession> =
        api_client(&url, "POST", NO_BODY).await?;
    Ok(map_single_resource(resp, map_personal_session_oneshot))
}

fn map_oauth2_session(
    resource: coauth_admin_types::SingleResource<coauth_admin_types::OAuthSession>,
) -> CoauthOAuth2Session {
    let attrs = resource.attributes;
    CoauthOAuth2Session {
        id: resource.id,
        user_id: attrs.user_id,
        client_id: Some(attrs.client_id).filter(|value| !value.is_empty()),
        scope: Some(attrs.scope).filter(|value| !value.is_empty()),
        human_name: attrs.human_name,
        created_at: Some(attrs.created_at.to_rfc3339()),
        finished_at: attrs.finished_at.map(|value| value.to_rfc3339()),
    }
}

fn map_personal_session(
    resource: coauth_admin_types::SingleResource<coauth_admin_types::PersonalSession>,
) -> CoauthPersonalSession {
    let attrs = resource.attributes;
    CoauthPersonalSessionRow {
        id: resource.id,
        user_id: attrs
            .actor_user_id
            .is_empty()
            .then(|| attrs.owner_user_id.clone())
            .flatten()
            .or_else(|| Some(attrs.actor_user_id).filter(|value| !value.is_empty())),
        name: Some(attrs.human_name).filter(|value| !value.is_empty()),
        created_at: Some(attrs.created_at.to_rfc3339()),
        last_active_at: attrs.last_active_at.map(|value| value.to_rfc3339()),
    }
}

fn map_personal_session_oneshot(
    resource: coauth_admin_types::SingleResource<coauth_admin_types::PersonalSession>,
) -> CoauthPersonalSessionOneShot {
    let access_token = resource.attributes.access_token.clone();
    CoauthPersonalSessionOneShot {
        session: map_personal_session(resource),
        access_token,
    }
}

#[cfg(test)]
mod tests {
    use super::{CoauthPersonalSessionOneShot, CoauthPersonalSessionRow};

    #[test]
    fn personal_session_row_drops_token_fields() {
        let row: CoauthPersonalSessionRow = serde_json::from_value(serde_json::json!({
            "id": "session-1",
            "user_id": "user-1",
            "name": "ops key",
            "created_at": "2026-06-07T00:00:00Z",
            "token": "secret-token",
            "access_token": "secret-access-token"
        }))
        .expect("row should deserialize while ignoring one-shot secrets");

        assert_eq!(row.id, "session-1");
        assert_eq!(row.user_id.as_deref(), Some("user-1"));
        assert_eq!(row.name.as_deref(), Some("ops key"));

        let serialized = serde_json::to_value(&row).expect("row serializes");
        assert!(serialized.get("token").is_none());
        assert!(serialized.get("access_token").is_none());
    }

    #[test]
    fn personal_session_one_shot_uses_current_access_token_field() {
        let response: CoauthPersonalSessionOneShot = serde_json::from_value(serde_json::json!({
            "id": "session-1",
            "name": "ops key",
            "access_token": "secret-access-token"
        }))
        .expect("one-shot response should deserialize");

        assert_eq!(response.session.name.as_deref(), Some("ops key"));
        assert_eq!(
            response.access_token.as_deref(),
            Some("secret-access-token")
        );
    }
}
