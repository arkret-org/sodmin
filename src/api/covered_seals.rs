//! HTTP client for the E2EE covered_seals lag admin describe endpoint
//! (Stream H', H'7).
//!
//! Endpoints:
//!
//! - `GET /_soland/admin/realms/{realm_id}/mls/covered-seals` — soland projects the covered-seals
//!   state for the `ck:cell:ck.component.covered_seals.v1:<realm_id>` cell along with the current
//!   governance Seal set so the admin can compute lag.
//! - `POST /_soland/admin/realms/{realm_id}/mls/covered-seals/advance` — admin override that asks
//!   the principal-server to manually fold the current governance Seal set into the covered_seals
//!   cell. Used when MLS members are offline and can't ack on their own; the override is a coarse
//!   hammer (it doesn't replace per-epoch MLS commits) so the page only surfaces it when lag >
//!   threshold. On 404 the UI surfaces a "not yet wired" toast.

use crate::api::client::{NO_BODY, api_client};
use crate::types::covered_seals::{CoveredSealsAdvanceOutcome, CoveredSealsSnapshot};
use crate::utils::net::error::HttpError;

/// Fetch the covered_seals snapshot for a Realm.
pub async fn get_covered_seals(realm_id: &str) -> Result<CoveredSealsSnapshot, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/mls/covered-seals",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", NO_BODY).await
}

/// Admin override: manually advance covered_seals so it matches the
/// current governance Seal set. soland constructs the appropriate or-set
/// Control Move, submits onto the canonical Control Move pipeline, and
/// returns the new lag count (typically 0).
pub async fn advance(realm_id: &str) -> Result<CoveredSealsAdvanceOutcome, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/mls/covered-seals/advance",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "POST", Some(&serde_json::json!({}))).await
}
