use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn list_reports(
    page: u64,
    per_page: u64,
    status: &str,
) -> Result<ListResponse<Report>, HttpError> {
    let url = build_url(
        "/_soland/admin/reports",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
            ("status", status),
        ],
    )?;
    api_client(&url, "GET", None).await
}

pub async fn get_report(id: &str) -> Result<Report, HttpError> {
    let url = format!("/_soland/admin/reports/{}", urlencoding::encode(id));
    api_client(&url, "GET", None).await
}

pub async fn update_report(id: &str, req: &UpdateReportRequest) -> Result<Report, HttpError> {
    let url = format!("/_soland/admin/reports/{}", urlencoding::encode(id));
    api_client(
        &url,
        "PATCH",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn delete_report(id: &str) -> Result<(), HttpError> {
    let url = format!("/_soland/admin/reports/{}", urlencoding::encode(id));
    api_client(&url, "DELETE", None).await
}
