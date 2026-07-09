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

use crate::api::client::{NO_BODY, api_client};
use crate::types::{RealmLinkDirection, RealmLinkList};
use crate::utils::net::error::HttpError;

pub type LinkDirection = RealmLinkDirection;

fn direction_query(direction: LinkDirection) -> String {
    serde_json::to_value(direction)
        .ok()
        .and_then(|value| value.as_str().map(ToOwned::to_owned))
        .unwrap_or_else(|| "both".to_owned())
}

pub async fn list_realm_links(
    realm_id: &str,
    direction: LinkDirection,
) -> Result<RealmLinkList, HttpError> {
    let url = format!(
        "/_arkret/self/realms/{}/links?direction={}",
        urlencoding::encode(realm_id),
        direction_query(direction),
    );
    api_client(&url, "GET", NO_BODY).await
}
