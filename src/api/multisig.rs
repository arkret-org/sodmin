//! HTTP client for read-only multi-signature aggregation status.
//!
//! Endpoints:
//!
//! - `GET  /_soland/admin/realms/{realm_id}/multisig/pending` — list pending Seals awaiting
//!   threshold (`k of n`). Each row includes the seal_id, threshold, collected partials count, and
//!   missing signers DIDs.

use crate::api::client::{NO_BODY, api_client};
use crate::types::multisig::{MultisigPendingOutcome, PendingMultisigSeal};
use crate::utils::net::error::HttpError;

/// List Seals awaiting partial signatures inside a Realm.
pub async fn list_pending(realm_id: &str) -> Result<Vec<PendingMultisigSeal>, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/multisig/pending",
        urlencoding::encode(realm_id)
    );
    let outcome: MultisigPendingOutcome = api_client(&url, "GET", NO_BODY).await?;
    Ok(outcome.entries)
}
