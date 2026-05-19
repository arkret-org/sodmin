//! HTTP client for the soland delivery-binding-policy admin surface
//! (T6.2 §3).
//!
//! The cell lives at `cx.cell.space.{space_id}.delivery_binding_policy`.
//! `allowed_recipient_services` and `binding_source_policy` are
//! operator-mutable; `policy_frontier` is reducer-owned and read-only
//! on the admin surface.

use crate::api::client::api_client;
use crate::types::*;
use crate::utils::error::HttpError;

pub async fn get_delivery_binding_policy(
    space_id: &str,
) -> Result<SpaceDeliveryBindingPolicy, HttpError> {
    let url = format!(
        "/api/admin/v1/spaces/{}/delivery-binding-policy",
        urlencoding::encode(space_id)
    );
    api_client(&url, "GET", None).await
}

pub async fn update_delivery_binding_policy(
    space_id: &str,
    req: &UpdateDeliveryBindingPolicyRequest,
) -> Result<SpaceDeliveryBindingPolicy, HttpError> {
    let url = format!(
        "/api/admin/v1/spaces/{}/delivery-binding-policy",
        urlencoding::encode(space_id)
    );
    api_client(
        &url,
        "PATCH",
        Some(serde_json::to_string(req).unwrap_or_default()),
    )
    .await
}

pub async fn list_member_routability(
    space_id: &str,
) -> Result<ListResponse<MemberRoutabilityRow>, HttpError> {
    let url = format!(
        "/api/admin/v1/spaces/{}/member-routability",
        urlencoding::encode(space_id)
    );
    api_client(&url, "GET", None).await
}
