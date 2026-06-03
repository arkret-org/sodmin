//! HTTP client for the multi-sig partial-signature aggregation admin
//! endpoints (Stream H', H'9).
//!
//! Endpoints:
//!
//! - `GET  /admin/spaces/{id}/multisig/pending` — list pending Anchors awaiting threshold
//!   (`k of n`). Each row includes the anchor_id, threshold, collected partials count, and missing
//!   signers DIDs.
//! - `POST /admin/spaces/{id}/multisig/{anchor_id}/partial` — submit the current admin's
//!   partial signature toward the pending Anchor. soland resolves the admin DID from the bearer
//!   token, signs the anchor's `state_root` with the bound signing key, and folds the resulting
//!   partial into the pending signature set.
//!
//! Both routes follow the 404-tolerant pattern.

use crate::api::client::api_client;
use crate::types::multisig::{
    PendingMultisigAnchor, SubmitPartialSignatureRequest, SubmitPartialSignatureResponse,
};
use crate::utils::error::HttpError;

/// List Anchors awaiting partial signatures inside a Space.
pub async fn list_pending(realm_id: &str) -> Result<Vec<PendingMultisigAnchor>, HttpError> {
    let url = format!(
        "/admin/spaces/{}/multisig/pending",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}

/// Submit the current admin's partial signature toward a pending Anchor.
pub async fn submit_partial(
    realm_id: &str,
    anchor_id: &str,
    note: Option<String>,
) -> Result<SubmitPartialSignatureResponse, HttpError> {
    let url = format!(
        "/admin/spaces/{}/multisig/{}/partial",
        urlencoding::encode(realm_id),
        urlencoding::encode(anchor_id),
    );
    let body = SubmitPartialSignatureRequest { note };
    api_client(
        &url,
        "POST",
        Some(serde_json::to_string(&body).unwrap_or_default()),
    )
    .await
}
