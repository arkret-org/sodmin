//! Realm organization control API client (SOD-ORG-01..03).
//!
//! The backing endpoints are not wired yet:
//!   * soland's verified Realm organization projection (SOL-ORG-06) for the
//!     relationship panel; and
//!   * coauth's organization-principal admin API (COA-ORG-05) for the
//!     delegation / PCR audit view.
//!
//! Until those land, these read paths return **empty** panels so the page
//! renders an explicit "feature not yet live" empty state. No fabricated
//! governance data is ever shown in production. When the real endpoints ship,
//! replace the empty bodies with `api_client` calls against
//! `/_soland/admin/...` (SOL-ORG-06) and `/_coauth/admin/...` (COA-ORG-05),
//! consuming `coauth_admin_types::organization_admin::OrganizationControlView`
//! and soland's verified projection rather than re-defining wire types here.

use crate::types::{OrgPrincipalControlPanel, RealmOrganizationPanel};
use crate::utils::net::error::HttpError;

/// SOD-ORG-01 — fetch the verified Realm organization relationship panel.
///
/// TODO(SOL-ORG-06): replace the empty body with a real GET against soland's
/// verified Realm organization projection, e.g.
/// `GET /_soland/admin/realms/{realm_id}/organizations` returning
/// `{ declared_owning_organizations: [Did], verified: [VerifiedOrgRelationship] }`.
/// soland's projection MUST already bucket each row's lifecycle
/// (active / revoked / expired / stale) — sodmin does not recompute it.
pub async fn get_realm_organization_panel(
    realm_id: &str,
) -> Result<RealmOrganizationPanel, HttpError> {
    // SOL-ORG-06 pending — return an empty panel (explicit empty state),
    // never fabricated rows.
    Ok(RealmOrganizationPanel {
        realm_id: realm_id.to_string(),
        declared_owning_organizations: Vec::new(),
        verified: Vec::new(),
    })
}

/// SOD-ORG-02 — fetch the organization-principal control / delegation audit
/// panel.
///
/// TODO(COA-ORG-05): replace the empty body with a real GET against coauth's
/// organization-principal admin API
/// (`GET /_coauth/admin/organizations/{organization_id}`), consuming
/// `coauth_admin_types::organization_admin::OrganizationControlView`.
pub async fn get_org_principal_control_panel(
    realm_id: &str,
) -> Result<OrgPrincipalControlPanel, HttpError> {
    // COA-ORG-05 pending — return an empty panel (explicit empty state),
    // never fabricated rows.
    Ok(OrgPrincipalControlPanel {
        realm_id: realm_id.to_string(),
        rows: Vec::new(),
    })
}
