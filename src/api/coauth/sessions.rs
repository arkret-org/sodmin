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
        "/_coauth/admin/oauth-sessions/{}/finish",
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

pub async fn create_personal_session(
    name: &str,
) -> Result<CoauthPersonalSessionOneShot, HttpError> {
    let body = serde_json::json!({ "name": name });
    api_client(
        "/_coauth/admin/personal-sessions",
        "POST",
        Some(body.to_string()),
    )
    .await
}

pub async fn revoke_personal_session(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_coauth/admin/personal-sessions/{}/revoke",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

pub async fn regenerate_personal_session(
    id: &str,
) -> Result<CoauthPersonalSessionOneShot, HttpError> {
    let url = format!(
        "/_coauth/admin/personal-sessions/{}/regenerate",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
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
