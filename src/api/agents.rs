use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn list_agents(page: u64, per_page: u64) -> Result<ListResponse<Agent>, HttpError> {
    let url = build_url(
        "/api/admin/v1/agents",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn get_agent(id: &str) -> Result<Agent, HttpError> {
    let url = format!("/api/admin/v1/agents/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

pub async fn disable_agent(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/agents/{}/disable",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}

pub async fn enable_agent(id: &str) -> Result<(), HttpError> {
    let url = format!(
        "/api/admin/v1/agents/{}/enable",
        urlencoding::encode(id)
    );
    api_client(&url, "POST", None).await
}
