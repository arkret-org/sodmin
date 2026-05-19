//! HTTP client for the soland delivery-binding-policy admin surface
//! (T6.2 §3, realm-rework).
//!
//! The cell lives at `cx.cell.realm.{realm_id}.delivery_binding_policy`.
//! `allowed_recipient_services` and `binding_source_policy` are
//! operator-mutable; `policy_frontier` is reducer-owned and read-only
//! on the admin surface.
//!
//! Realm-rework: the admin endpoint moved from
//! `/api/admin/v1/spaces/{id}/delivery-binding-policy` to
//! `/api/admin/v1/realms/{id}/delivery-binding-policy` because the
//! security boundary is now spelled "Realm".

use crate::api::client::api_client;
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn get_delivery_binding_policy(
    realm_id: &str,
) -> Result<RealmDeliveryBindingPolicy, HttpError> {
    let url = format!(
        "/api/admin/v1/realms/{}/delivery-binding-policy",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}

pub async fn update_delivery_binding_policy(
    realm_id: &str,
    req: &UpdateDeliveryBindingPolicyRequest,
) -> Result<RealmDeliveryBindingPolicy, HttpError> {
    let url = format!(
        "/api/admin/v1/realms/{}/delivery-binding-policy",
        urlencoding::encode(realm_id)
    );
    api_client(
        &url,
        "PATCH",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn list_member_routability(
    realm_id: &str,
) -> Result<ListResponse<MemberRoutabilityRow>, HttpError> {
    // TODO(realm-rework): once soland exposes a Realm-scoped
    // `/api/admin/v1/realms/{id}/member-routability` route, switch to
    // it. Until then this remains the Space-scoped path (the row is
    // the same — security-boundary members keyed by what we now call
    // a Realm).
    let url = format!(
        "/api/admin/v1/realms/{}/member-routability",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}
