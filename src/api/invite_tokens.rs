use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::net::error::HttpError;

pub async fn list_invite_tokens(
    page: u64,
    per_page: u64,
) -> Result<ListResponse<InviteToken>, HttpError> {
    let url = build_url(
        "/_soland/admin/invite-tokens",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn create_invite_token(req: &CreateInviteTokenRequest) -> Result<InviteToken, HttpError> {
    api_client(
        "/_soland/admin/invite-tokens",
        "POST",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn delete_invite_token(id: &str) -> Result<(), HttpError> {
    let url = format!("/_soland/admin/invite-tokens/{}", urlencoding::encode(id));
    api_client(&url, "DELETE", None).await
}
