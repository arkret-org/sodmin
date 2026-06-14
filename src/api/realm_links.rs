//! HTTP client for the soland Realm link-graph admin surface
//! (R5.2, Round R1.2 — `ck.realm.link` projection).
//!
//! A "Realm link" is a typed edge between two security boundaries.
//! Examples: `governed_by` (parent for capability inheritance),
//! `discoverable_from` (cross-realm directory hop), `mirror_of`
//! (federated mirror of another Realm).
//!
//! The admin surface returns the outbound and/or inbound edge list
//! for a single Realm so the operator can see who governs / mirrors /
//! discovers them and vice-versa. The full graph view is a future
//! enhancement; for now the page renders the rows as a simple list of
//! chips.

use crate::api::client::api_client;
use crate::types::{RealmLinkDirection, RealmLinkList};
use crate::utils::net::error::HttpError;

pub type LinkDirection = RealmLinkDirection;

fn direction_query(direction: LinkDirection) -> &'static str {
    match direction {
        RealmLinkDirection::Outbound => "outbound",
        RealmLinkDirection::Inbound => "inbound",
        RealmLinkDirection::Both => "both",
    }
}

pub async fn list_realm_links(
    realm_id: &str,
    direction: LinkDirection,
) -> Result<RealmLinkList, HttpError> {
    let url = format!(
        "/_cokret/self/realms/{}/links?direction={}",
        urlencoding::encode(realm_id),
        direction_query(direction),
    );
    api_client(&url, "GET", None).await
}
