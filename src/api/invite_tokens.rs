use crate::api::client::{NO_BODY, NoBody, api_client, build_url};
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
    api_client(&url, "GET", NO_BODY).await
}

pub async fn create_invite_token(req: &CreateInviteTokenRequest) -> Result<InviteToken, HttpError> {
    api_client("/_soland/admin/invite-tokens", "POST", Some(req)).await
}

pub async fn revoke_invite_token(id: &str) -> Result<(), HttpError> {
    let url = format!("/_soland/admin/invite-tokens/{}", urlencoding::encode(id));
    let _: NoBody = api_client(&url, "DELETE", NO_BODY).await?;
    Ok(())
}
