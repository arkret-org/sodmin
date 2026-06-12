//! HTTP client for the soland CKP-0007 Circle admin surface
//! (`/_soland/self/circles/*`).
//!
//! Routes consumed:
//!
//! - `GET    /_soland/self/circles?realm_id=...`              list Circles in a Realm
//! - `GET    /_soland/self/circles/{circle_id}`               read Circle
//! - `POST   /_soland/self/circles`                           create Circle
//! - `POST   /_soland/self/circles/{circle_id}/members`       add / re-state member
//! - `DELETE /_soland/self/circles/{circle_id}/members/{actor_id}` remove member
//! - `POST   /_soland/self/circles/{circle_id}/scope-rotate`  rotate the MLS scope
//! - `POST   /_soland/self/circles/{circle_id}/archive`       archive
//! - `POST   /_soland/self/circles/{circle_id}/tombstone`     tombstone
//!
//! The reducer surfaces canonical CKP-0007 reason codes (e.g.
//! `circle_realm_mismatch`, `circle_member_must_be_realm_member`,
//! `circle_not_active`) via the standard `AppError.code` field, which the
//! admin UI maps to localised strings in `utils::net::error`.

use crate::api::client::{api_client, build_url, json_body};
use crate::types::circles::{
    Circle, CircleMemberRequest, CircleMembershipOutcome, CircleScopeRotateOutcome,
    CreateCircleRequest, ListCirclesOutcome,
};
use crate::utils::net::error::HttpError;

/// List Circles inside a Realm. Requires `ck.circle.audit` to enumerate
/// outside the caller's own membership; soland enforces the visibility
/// filter server-side.
pub async fn list_circles(realm_id: &str) -> Result<ListCirclesOutcome, HttpError> {
    let url = build_url("/_soland/self/circles", &[("realm_id", realm_id)])?;
    api_client(&url, "GET", None).await
}

/// Read a single Circle by id.
pub async fn get_circle(circle_id: &str) -> Result<Circle, HttpError> {
    let url = format!("/_soland/self/circles/{}", urlencoding::encode(circle_id));
    api_client(&url, "GET", None).await
}

/// Create a Circle inside the given Realm. Requires `ck.circle.create`.
pub async fn create_circle(req: &CreateCircleRequest) -> Result<Circle, HttpError> {
    let body = json_body(req)?;
    api_client("/_soland/self/circles", "POST", Some(body)).await
}

/// Add a member (or transition member state) inside the Circle.
/// Defaults to `active` server-side when `state` is `None`. Requires
/// `ck.circle.member.add` for self-join or `ck.circle.member.add.others`
/// when `actor_id != session.actor`. The strict-subset invariant
/// (member must already be a Realm member) is enforced by the reducer.
pub async fn add_circle_member(
    circle_id: &str,
    req: &CircleMemberRequest,
) -> Result<CircleMembershipOutcome, HttpError> {
    let url = format!(
        "/_soland/self/circles/{}/members",
        urlencoding::encode(circle_id),
    );
    let body = json_body(req)?;
    api_client(&url, "POST", Some(body)).await
}

/// Remove a member (transitions to `removed`).
pub async fn remove_circle_member(
    circle_id: &str,
    actor_id: &str,
) -> Result<CircleMembershipOutcome, HttpError> {
    let url = format!(
        "/_soland/self/circles/{}/members/{}",
        urlencoding::encode(circle_id),
        urlencoding::encode(actor_id),
    );
    api_client(&url, "DELETE", None).await
}

/// Rotate the Circle's bound MLS group. CKP-0007 mandates this be a
/// separate explicit admin action so the receipt fans out into the
/// audit log even when no membership changes accompany the rotation.
pub async fn rotate_circle_scope(circle_id: &str) -> Result<CircleScopeRotateOutcome, HttpError> {
    let url = format!(
        "/_soland/self/circles/{}/scope-rotate",
        urlencoding::encode(circle_id),
    );
    api_client(&url, "POST", Some("{}".to_string())).await
}

/// Archive the Circle (`ck.circle.archive`). Reversible by the same
/// caller while the Circle is still inside the soft-delete window.
pub async fn archive_circle(circle_id: &str) -> Result<Circle, HttpError> {
    let url = format!(
        "/_soland/self/circles/{}/archive",
        urlencoding::encode(circle_id),
    );
    api_client(&url, "POST", Some("{}".to_string())).await
}

/// Tombstone the Circle (`ck.circle.tombstone`). Terminal state; no
/// further admin actions accepted.
pub async fn tombstone_circle(circle_id: &str) -> Result<Circle, HttpError> {
    let url = format!(
        "/_soland/self/circles/{}/tombstone",
        urlencoding::encode(circle_id),
    );
    api_client(&url, "POST", Some("{}".to_string())).await
}
