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
use crate::types::{ListResponse, RealmLinkRow};
use crate::utils::error::HttpError;

/// Direction of the link list query. The wire surface accepts
/// `outbound` (this Realm → others) or `inbound` (others → this Realm).
#[derive(Debug, Clone, Copy)]
pub enum LinkDirection {
    Outbound,
    Inbound,
}

impl LinkDirection {
    fn as_query(self) -> &'static str {
        match self {
            LinkDirection::Outbound => "outbound",
            LinkDirection::Inbound => "inbound",
        }
    }
}

pub async fn list_realm_links(
    realm_id: &str,
    direction: LinkDirection,
) -> Result<ListResponse<RealmLinkRow>, HttpError> {
    let url = format!(
        "/_soland/admin/realms/{}/links?direction={}",
        urlencoding::encode(realm_id),
        direction.as_query(),
    );
    api_client(&url, "GET", None).await
}
