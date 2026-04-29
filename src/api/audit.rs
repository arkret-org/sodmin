use crate::api::client::{api_client, build_url};
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn list_audit_entries(
    page: u64,
    per_page: u64,
) -> Result<ListResponse<AuditEntry>, HttpError> {
    let url = build_url(
        "/_cx/admin/v1/audit",
        &[
            ("page", &page.to_string()),
            ("per_page", &per_page.to_string()),
        ],
    )?;
    api_client(&url, "GET", None).await
}
