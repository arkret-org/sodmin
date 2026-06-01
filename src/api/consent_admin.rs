//! HTTP client for the consent cell admin describe endpoint (Stream H',
//! H'5).
//!
//! Endpoints:
//!
//! - `GET /api/admin/v1/spaces/{id}/consent` — read the joined or-set value of
//!   `cx:cell:cx.component.consent.v1:<holder_did>` for every holder visible inside the Space.
//!   soland is responsible for redaction: it MUST NOT expose holder-private peer relations beyond
//!   the public admin-visible projection (DID / peer DID / scope / status / created_at).
//! - `POST /api/admin/v1/consent/{consent_id}/resolve` — admin override for **pending** consent
//!   rows. Same shape as the invite-quarantine resolve route: `{decision: approve|reject, note?}`.
//!   soland is expected to validate that the consent row is currently `Pending` and reject
//!   otherwise. The endpoint may not yet be wired on the backend; the caller surfaces a "not yet
//!   wired" toast on 404 (see `pages/spaces/consent.rs`).
//!
//! The describe surface is intentionally read-only beyond this admin
//! resolve override — admins can't grant arbitrary consent on behalf of
//! a holder (the consent capability is bound to the holder's signing
//! key). Resolve only exists for the narrow case of stuck `Pending` rows
//! that need an operator decision.

use crate::api::client::api_client;
use crate::types::consent::{ConsentGrant, ConsentResolveDecision, ConsentResolveRequest};
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

/// Resolve a pending consent row by admin decision.
///
/// `POST /api/admin/v1/consent/{consent_id}/resolve` with body
/// `{decision: approve|reject, note?}`. Mirrors the invite-quarantine
/// resolve route shape that soland already implements; the returned
/// envelope is the same shape — a single resource describing the
/// resolved row's new status.
///
/// On 404 (endpoint not yet wired on the backend) the caller is
/// expected to surface a clear toast rather than a generic error so
/// operators can tell "feature not deployed" apart from "row missing".
pub async fn resolve(
    consent_id: &str,
    decision: ConsentResolveDecision,
    note: Option<String>,
) -> Result<ConsentGrant, HttpError> {
    let url = format!(
        "/api/admin/v1/consent/{}/resolve",
        urlencoding::encode(consent_id)
    );
    let body = ConsentResolveRequest { decision, note };
    api_client(
        &url,
        "POST",
        Some(serde_json::to_string(&body).unwrap_or_default()),
    )
    .await
}
