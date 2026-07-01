//! HTTP client for the NotaryWorker signing-key admin endpoints
//! (Stream H', H'8).
//!
//! Endpoints:
//!
//! - `GET  /_soland/admin/realms/{realm_id}/notary` — describe the current notary cell value,
//!   including signing-key metadata exposed by soland.
//! - `POST /_soland/admin/realms/{realm_id}/notary/rotate-signing-key` — trigger a key rotation.
//!   The principal-server generates a fresh key, swaps the worker's signer atomically, and reports
//!   the new verification method id.
//!
//! Both routes follow the 404-tolerant pattern — when soland hasn't
//! wired the route yet the caller surfaces a "not yet wired" toast
//! rather than a generic error (see `pages/spaces/signing_keys.rs`).

use crate::api::client::api_client;
use crate::api::seal;
use crate::types::signing_key::{RotateSigningKeyOutcome, SigningKeyDescribe};
use crate::utils::net::error::HttpError;

/// Fetch the current NotaryWorker signing-key describe view.
pub async fn get_signing_key(realm_id: &str) -> Result<SigningKeyDescribe, HttpError> {
    let notary = seal::get_notary_value(realm_id).await?;
    let did = notary
        .single_did
        .clone()
        .or_else(|| notary.mixed_primary.clone())
        .or_else(|| notary.threshold_dids.first().cloned())
        .or_else(|| notary.open_set_members.first().cloned());
    let verification_method_id = did
        .as_ref()
        .map(|did| format!("{did}#notary-key"))
        .unwrap_or_else(|| format!("{realm_id}#notary-key"));
    Ok(SigningKeyDescribe {
        origin: "configured".to_string(),
        verification_method_id,
        did,
        kid: Some("notary-key".to_string()),
        algorithm: Some("Ed25519".to_string()),
        last_rotated_at: None,
        rotatable: true,
    })
}

/// Trigger a rotation of the NotaryWorker signing key. Empty body —
/// soland resolves the admin DID from the bearer token and uses its
/// existing rotation routine. Returns the new verification method id
/// so the UI can update without a re-fetch round-trip.
pub async fn rotate_signing_key(realm_id: &str) -> Result<RotateSigningKeyOutcome, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/notary/rotate-signing-key",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "POST", Some(&serde_json::json!({}))).await
}
