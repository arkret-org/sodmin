//! HTTP client for the consent cell admin describe endpoint (Stream H',
//! H'5).
//!
//! Endpoint:
//!
//! - `GET /api/admin/v1/spaces/{id}/consent` — read the joined or-set
//!   value of `cx:cell:cx.component.consent.v1:<holder_did>` for every
//!   holder visible inside the Space. soland is responsible for redaction:
//!   it MUST NOT expose holder-private peer relations beyond the public
//!   admin-visible projection (DID / peer DID / scope / status / created_at).
//!
//! Read-only on the sodmin side; admins can't grant on behalf of users
//! (would violate the consent capability — the holder's signing key is
//! the only authorized signer).

use crate::api::client::api_client;
use crate::types::consent::ConsentGrant;
use crate::utils::error::HttpError;

/// Fetch the list of consent grants visible inside the Space.
///
/// `GET /api/admin/v1/spaces/{id}/consent`. soland projects each consent
/// cell's joined or-set value into one row per (holder, peer, scope)
/// triple.
pub async fn list_consent_grants(space_id: &str) -> Result<Vec<ConsentGrant>, HttpError> {
    let url = format!(
        "/api/admin/v1/spaces/{}/consent",
        urlencoding::encode(space_id)
    );
    api_client(&url, "GET", None).await
}
