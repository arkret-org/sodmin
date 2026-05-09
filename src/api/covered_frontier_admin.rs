//! HTTP client for the E2EE covered_frontier lag admin describe endpoint
//! (Stream H', H'7).
//!
//! Endpoint:
//!
//! - `GET /api/admin/v1/spaces/{id}/mls/covered-frontier` — soland projects
//!   the lattice or-set state for the
//!   `cx:cell:cx.component.mls.covered_frontier.v1:<space_id>` cell along
//!   with the current governance frontier so the admin can compute lag.

use crate::api::client::api_client;
use crate::types::covered_frontier::CoveredFrontierSnapshot;
use crate::utils::error::HttpError;

/// Fetch the covered_frontier snapshot for a Space.
pub async fn get_covered_frontier(space_id: &str) -> Result<CoveredFrontierSnapshot, HttpError> {
    let url = format!(
        "/api/admin/v1/spaces/{}/mls/covered-frontier",
        urlencoding::encode(space_id)
    );
    api_client(&url, "GET", None).await
}
