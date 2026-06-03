use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn list_applets(page: u64, per_page: u64) -> Result<ListResponse<Applet>, HttpError> {
    let url = build_url(
        "/admin/applets",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn register_applet(req: &RegisterAppletRequest) -> Result<Applet, HttpError> {
    api_client(
        "/admin/applets",
        "POST",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn delete_applet(id: &str) -> Result<(), HttpError> {
    let url = format!("/admin/applets/{}", urlencoding::encode(id));
    api_client(&url, "DELETE", None).await
}

pub async fn enable_applet(id: &str) -> Result<(), HttpError> {
    let url = format!("/admin/applets/{}/enable", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}

pub async fn disable_applet(id: &str) -> Result<(), HttpError> {
    let url = format!("/admin/applets/{}/disable", urlencoding::encode(id));
    api_client(&url, "POST", None).await
}
