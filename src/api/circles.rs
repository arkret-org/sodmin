//! HTTP client for the soland CXP-0007 Circle admin surface
//! (`/api/v1/circles/*`).
//!
//! Routes consumed:
//!
//! - `GET    /api/v1/circles?realm_id=...`              list Circles in a Realm
//! - `GET    /api/v1/circles/{circle_id}`               read Circle
//! - `POST   /api/v1/circles`                           create Circle
//! - `POST   /api/v1/circles/{circle_id}/members`       add / re-state member
//! - `DELETE /api/v1/circles/{circle_id}/members/{actor_did}` remove member
//! - `POST   /api/v1/circles/{circle_id}/scope-rotate`  rotate the MLS scope
//! - `POST   /api/v1/circles/{circle_id}/archive`       archive
//! - `POST   /api/v1/circles/{circle_id}/tombstone`     tombstone
//!
//! The reducer surfaces canonical CXP-0007 reason codes (e.g.
//! `circle_realm_mismatch`, `circle_member_must_be_realm_member`,
//! `circle_not_active`) via the standard `AppError.code` field, which the
//! admin UI maps to localised strings in `utils::error`.

use crate::api::client::{api_client, build_url};
use crate::types::circles::{
    Circle, CircleMemberRequest, CircleMembershipResponse, CircleScopeRotateResponse,
    CreateCircleRequest, ListCirclesResponse,
};
use crate::utils::error::HttpError;

/// List Circles inside a Realm. Requires `cx.circle.audit` to enumerate
/// outside the caller's own membership; soland enforces the visibility
/// filter server-side.
pub async fn list_circles(realm_id: &str) -> Result<ListCirclesResponse, HttpError> {
    let url = build_url("/api/v1/circles", &[("realm_id", realm_id)])?;
    api_client(&url, "GET", None).await
}

/// Read a single Circle by id.
pub async fn get_circle(circle_id: &str) -> Result<Circle, HttpError> {
    let url = format!("/api/v1/circles/{}", urlencoding::encode(circle_id));
    api_client(&url, "GET", None).await
}

/// Create a Circle inside the given Realm. Requires `cx.circle.create`.
pub async fn create_circle(req: &CreateCircleRequest) -> Result<Circle, HttpError> {
    let body = serde_json::to_string(req).unwrap_or_default();
    api_client("/api/v1/circles", "POST", Some(body)).await
}

/// Add a member (or transition member state) inside the Circle.
/// Defaults to `active` server-side when `state` is `None`. Requires
/// `cx.circle.member.add` for self-join or `cx.circle.member.add.others`
/// when `actor_did != session.actor`. The strict-subset invariant
/// (member must already be a Realm member) is enforced by the reducer.
pub async fn add_circle_member(
    circle_id: &str,
    req: &CircleMemberRequest,
) -> Result<CircleMembershipResponse, HttpError> {
    let url = format!(
        "/api/v1/circles/{}/members",
        urlencoding::encode(circle_id),
    );
    let body = serde_json::to_string(req).unwrap_or_default();
    api_client(&url, "POST", Some(body)).await
}

/// Remove a member (transitions to `removed`).
pub async fn remove_circle_member(
    circle_id: &str,
    actor_did: &str,
) -> Result<CircleMembershipResponse, HttpError> {
    let url = format!(
        "/api/v1/circles/{}/members/{}",
        urlencoding::encode(circle_id),
        urlencoding::encode(actor_did),
    );
    api_client(&url, "DELETE", None).await
}

/// Rotate the Circle's bound MLS group. CXP-0007 mandates this be a
/// separate explicit admin action so the receipt fans out into the
/// audit log even when no membership changes accompany the rotation.
pub async fn rotate_circle_scope(
    circle_id: &str,
) -> Result<CircleScopeRotateResponse, HttpError> {
    let url = format!(
        "/api/v1/circles/{}/scope-rotate",
        urlencoding::encode(circle_id),
    );
    api_client(&url, "POST", Some("{}".to_string())).await
}

/// Archive the Circle (`cx.circle.archive`). Reversible by the same
/// caller while the Circle is still inside the soft-delete window.
pub async fn archive_circle(circle_id: &str) -> Result<Circle, HttpError> {
    let url = format!(
        "/api/v1/circles/{}/archive",
        urlencoding::encode(circle_id),
    );
    api_client(&url, "POST", Some("{}".to_string())).await
}

/// Tombstone the Circle (`cx.circle.tombstone`). Terminal state; no
/// further admin actions accepted.
pub async fn tombstone_circle(circle_id: &str) -> Result<Circle, HttpError> {
    let url = format!(
        "/api/v1/circles/{}/tombstone",
        urlencoding::encode(circle_id),
    );
    api_client(&url, "POST", Some("{}".to_string())).await
}
