use crate::api::client::{api_client, build_url, json_body};
use crate::types::*;
use crate::utils::net::error::HttpError;

pub async fn list_invite_tokens(
    page: u64,
    per_page: u64,
) -> Result<ListResponse<InviteToken>, HttpError> {
    let limit = per_page.max(1);
    let cursor = page.saturating_sub(1).saturating_mul(limit).to_string();
    let url = build_url(
        "/_soland/admin/invite-tokens",
        &[("limit", &limit.to_string()), ("cursor", &cursor)],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn create_invite_token(req: &CreateInviteTokenRequest) -> Result<InviteToken, HttpError> {
    api_client(
        "/_soland/admin/invite-tokens",
        "POST",
        Some(json_body(req)?),
    )
    .await
}

pub async fn delete_invite_token(id: &str) -> Result<(), HttpError> {
    let url = format!("/_soland/admin/invite-tokens/{}", urlencoding::encode(id));
    let _: serde_json::Value = api_client(&url, "DELETE", None).await?;
    Ok(())
}
