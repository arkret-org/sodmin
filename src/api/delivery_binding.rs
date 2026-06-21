//! HTTP client for the soland delivery-binding-policy admin surface.
//!
//! The cell family is `ck.component.realm.delivery_binding_policy.v1`.
//! This surface is **read-only** in sodmin: realm policy writes are a
//! general-management action that strands through events / yougen, not the
//! operations console. sodmin only renders the effective policy
//! (`allowed_recipient_services`, `binding_source_policy`,
//! reducer-owned `policy_frontier`) plus the member-routability and
//! handover diagnostics.
//!
//! The admin endpoint is
//! `/_soland/admin/realms/{id}/delivery-binding-policy` (GET only).

use crate::api::client::api_client;
use crate::types::*;
use crate::utils::net::error::HttpError;

pub async fn get_delivery_binding_policy(
    realm_id: &str,
) -> Result<RealmDeliveryBindingPolicy, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/delivery-binding-policy",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}

pub async fn list_member_routability(
    realm_id: &str,
) -> Result<ListResponse<MemberRoutabilityRow>, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/member-routability",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}

pub async fn list_delivery_binding_handovers(
    realm_id: &str,
) -> Result<ListResponse<DeliveryBindingHandoverRow>, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/delivery-binding/handovers",
        urlencoding::encode(realm_id)
    );
    api_client(&url, "GET", None).await
}
