//! HTTP client for the AnchorerWorker signing-key admin endpoints
//! (Stream H', H'8).
//!
//! Endpoints:
//!
//! - `GET  /_soland/admin/realms/{realm_id}/anchorer/signing-key` — describe the current
//!   AnchorerWorker signing key (origin, DID#kid, last rotation).
//! - `POST /_soland/admin/realms/{realm_id}/anchorer/rotate-signing-key` — trigger a key rotation.
//!   The principal-server generates a fresh key, swaps the worker's signer atomically, and reports
//!   the new verification method id.
//!
//! Both routes follow the 404-tolerant pattern — when soland hasn't
//! wired the route yet the caller surfaces a "not yet wired" toast
//! rather than a generic error (see `pages/spaces/signing_keys.rs`).

use crate::api::client::api_client;
use crate::types::signing_key::{RotateSigningKeyResponse, SigningKeyDescribe};
use crate::utils::net::error::HttpError;

/// Fetch the current AnchorerWorker signing-key describe view.
pub async fn get_signing_key(realm_id: &str) -> Result<SigningKeyDescribe, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/anchorer/signing-key",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}

/// Trigger a rotation of the AnchorerWorker signing key. Empty body —
/// soland resolves the admin DID from the bearer token and uses its
/// existing rotation routine. Returns the new verification method id
/// so the UI can update without a re-fetch round-trip.
pub async fn rotate_signing_key(realm_id: &str) -> Result<RotateSigningKeyResponse, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/anchorer/rotate-signing-key",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "POST", Some("{}".to_string())).await
}
