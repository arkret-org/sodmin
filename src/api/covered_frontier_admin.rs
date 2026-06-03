//! HTTP client for the E2EE covered_frontier lag admin describe endpoint
//! (Stream H', H'7).
//!
//! Endpoints:
//!
//! - `GET /_soland/admin/spaces/{id}/mls/covered-frontier` — soland projects the lattice or-set
//!   state for the `ck:cell:cx.component.mls.covered_frontier.v1:<realm_id>` cell along with the
//!   current governance frontier so the admin can compute lag.
//! - `POST /_soland/admin/spaces/{id}/mls/covered-frontier/advance` — admin override that asks the
//!   principal-server to manually fold the current `governance_frontier` into the covered_frontier
//!   or-set. Used when MLS members are offline and can't ack on their own; the override is a coarse
//!   hammer (it doesn't replace per-epoch MLS commits) so the page only surfaces it when lag >
//!   threshold. On 404 the UI surfaces a "not yet wired" toast.

use crate::api::client::api_client;
use crate::types::covered_frontier::{CoveredFrontierAdvanceResponse, CoveredFrontierSnapshot};
use crate::utils::net::error::HttpError;

/// Fetch the covered_frontier snapshot for a Space.
pub async fn get_covered_frontier(realm_id: &str) -> Result<CoveredFrontierSnapshot, HttpError> {
    let url = format!(
        "/_soland/admin/spaces/{}/mls/covered-frontier",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}

/// Admin override: manually advance the covered_frontier so it matches
/// the current governance frontier. soland constructs the appropriate
/// or-set Move (signed by the admin's bearer token), submits onto the
/// canonical Move pipeline, and returns the new lag count (typically 0).
pub async fn advance(realm_id: &str) -> Result<CoveredFrontierAdvanceResponse, HttpError> {
    let url = format!(
        "/_soland/admin/spaces/{}/mls/covered-frontier/advance",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "POST", Some("{}".to_string())).await
}
