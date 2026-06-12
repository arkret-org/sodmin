//! HTTP client for the multi-sig partial-signature aggregation admin
//! endpoints (Stream H', H'9).
//!
//! Endpoints:
//!
//! - `GET  /_soland/admin/realms/{realm_id}/multisig/pending` — list pending Seals awaiting
//!   threshold (`k of n`). Each row includes the seal_id, threshold, collected partials count, and
//!   missing signers DIDs.
//! - `POST /_soland/admin/realms/{realm_id}/multisig/{seal_id}/partial` — submit the current
//!   admin's partial signature toward the pending Seal. soland resolves the admin DID from the
//!   bearer token, signs the Seal's `state_root` with the bound signing key, and folds the
//!   resulting partial into the pending signature set.
//!
//! Both routes follow the 404-tolerant pattern.

use crate::api::client::{api_client, json_body};
use crate::types::multisig::{
    MultisigPendingOutcome, PendingMultisigSeal, SubmitPartialSignatureOutcome,
    SubmitPartialSignatureRequest,
};
use crate::utils::net::error::HttpError;

/// List Seals awaiting partial signatures inside a Realm.
pub async fn list_pending(realm_id: &str) -> Result<Vec<PendingMultisigSeal>, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/multisig/pending",
        urlencoding::encode(realm_id)
    );
    let outcome: MultisigPendingOutcome = api_client(&url, "GET", None).await?;
    Ok(outcome.entries)
}

/// Submit the current admin's partial signature toward a pending Seal.
pub async fn submit_partial(
    realm_id: &str,
    seal_id: &str,
    note: Option<String>,
) -> Result<SubmitPartialSignatureOutcome, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/multisig/{}/partial",
        urlencoding::encode(realm_id),
        urlencoding::encode(seal_id),
    );
    let body = SubmitPartialSignatureRequest { note };
    api_client(&url, "POST", Some(json_body(&body)?)).await
}
