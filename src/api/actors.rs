use crate::api::client::{api_client, build_url, NoBody, NO_BODY};
use crate::types::*;
use crate::utils::net::error::HttpError;

pub async fn list_actors(
    page: u64,
    per_page: u64,
    search: &str,
) -> Result<ListResponse<Actor>, HttpError> {
    let limit = per_page.max(1);
    let cursor = page.saturating_sub(1).saturating_mul(limit).to_string();
    let url = build_url(
        "/_soland/admin/actors",
        &[("limit", &limit.to_string()), ("cursor", &cursor)],
    )?;
    let mut resp: ListResponse<Actor> = api_client(&url, "GET", NO_BODY).await?;
    let needle = search.trim().to_ascii_lowercase();
    if !needle.is_empty() {
        resp.data.retain(|actor| {
            actor.id.to_ascii_lowercase().contains(&needle)
                || actor.did.to_ascii_lowercase().contains(&needle)
                || actor
                    .handle
                    .as_deref()
                    .is_some_and(|value| value.to_ascii_lowercase().contains(&needle))
                || actor
                    .display_name
                    .as_deref()
                    .is_some_and(|value| value.to_ascii_lowercase().contains(&needle))
        });
    }
    Ok(resp)
}

pub async fn get_actor(id: &str) -> Result<Actor, HttpError> {
    let url = format!("/_soland/admin/actors/{}", urlencoding::encode(id));
    api_client(&url, "GET", NO_BODY).await
}

pub async fn deactivate_account(account_id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/_soland/admin/accounts/{}/deactivate",
        urlencoding::encode(account_id)
    );
    let _: NoBody = api_client(&url, "POST", NO_BODY).await?;
    Ok(())
}
