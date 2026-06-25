//! Realm organization control API client (SOD-ORG-01..03).
//!
//! This module is a **mock-backed stub today**. sodmin must consume:
//!   * soland's verified Realm organization projection (SOL-ORG-06) for the
//!     relationship panel; and
//!   * coauth's organization-principal admin API (COA-ORG-05) for the
//!     delegation / PCR audit view.
//!
//! Neither endpoint is defined yet, so the read paths below return DTOs
//! constructed from the canonical SDK `RealmOrganization*` types. When the real
//! endpoints land, replace the mock bodies with `api_client` calls against
//! `/_soland/admin/...` (SOL-ORG-06) and `/_coauth/admin/...` (COA-ORG-05);
//! the DTO shapes are intentionally close to the wire projection so the page
//! does not need to change.

use crate::types::{
    DelegationLifecycle, OrgPrincipalControl, OrgPrincipalControlPanel, RealmOrganizationControlScope,
    RealmOrganizationIssuerRole, RealmOrganizationPanel, RealmOrganizationRelationship,
    RelationshipLifecycle, VerifiedOrgRelationship,
};
use crate::utils::net::error::HttpError;

/// SOD-ORG-01 — fetch the verified Realm organization relationship panel.
///
/// TODO(SOL-ORG-06): replace the mock body with a real GET against soland's
/// verified Realm organization projection, e.g.
/// `GET /_soland/admin/realms/{realm_id}/organizations` returning
/// `{ declared_owning_organizations: [Did], verified: [VerifiedOrgRelationship] }`.
/// soland's projection MUST already bucket each row's lifecycle
/// (active / revoked / expired / stale) — sodmin does not recompute it.
pub async fn get_realm_organization_panel(
    realm_id: &str,
) -> Result<RealmOrganizationPanel, HttpError> {
    // --- MOCK (SOL-ORG-06 pending) ---
    Ok(mock_panel(realm_id))
}

/// SOD-ORG-02 — fetch the organization-principal control / delegation audit
/// panel.
///
/// TODO(COA-ORG-05): replace the mock body with a real GET against coauth's
/// organization-principal admin API, e.g.
/// `GET /_coauth/admin/organizations/{organization_id}/control` returning the
/// controller / governance-service / Account-Authority delegation / PCR
/// bootstrap source / `executed_by` projection.
pub async fn get_org_principal_control_panel(
    realm_id: &str,
) -> Result<OrgPrincipalControlPanel, HttpError> {
    // --- MOCK (COA-ORG-05 pending) ---
    Ok(mock_principal_control(realm_id))
}

// ── Mock fixtures (removed once SOL-ORG-06 / COA-ORG-05 ship) ──

fn mock_panel(realm_id: &str) -> RealmOrganizationPanel {
    let verified_org = "did:web:acme.example".to_string();
    let declared_only = "did:web:claimed-but-unconfirmed.example".to_string();
    let revoked_org = "did:web:former-sponsor.example".to_string();

    RealmOrganizationPanel {
        realm_id: realm_id.to_string(),
        // The Realm self-declares two orgs; only one has verified consent —
        // the other is declared-only (no organization statement).
        declared_owning_organizations: vec![verified_org.clone(), declared_only.clone()],
        verified: vec![
            VerifiedOrgRelationship {
                organization_id: verified_org,
                relationship: RealmOrganizationRelationship::Owner,
                control_scopes: vec![
                    RealmOrganizationControlScope::OfficialBadge,
                    RealmOrganizationControlScope::RealmAdmin,
                    RealmOrganizationControlScope::DirectoryListing,
                ],
                issued_at: "2026-01-10T08:00:00Z".to_string(),
                not_before: None,
                expires_at: Some("2027-01-10T08:00:00Z".to_string()),
                statement_id: "org-stmt-0001".to_string(),
                supersedes_statement_id: None,
                revokes_statement_id: None,
                lifecycle: RelationshipLifecycle::Active,
                issuer_role: RealmOrganizationIssuerRole::OrganizationDid,
            },
            VerifiedOrgRelationship {
                organization_id: revoked_org,
                relationship: RealmOrganizationRelationship::Sponsor,
                control_scopes: vec![RealmOrganizationControlScope::OfficialBadge],
                issued_at: "2025-06-01T08:00:00Z".to_string(),
                not_before: None,
                expires_at: Some("2026-06-01T08:00:00Z".to_string()),
                statement_id: "org-stmt-0009".to_string(),
                supersedes_statement_id: None,
                revokes_statement_id: Some("org-stmt-0002".to_string()),
                lifecycle: RelationshipLifecycle::Revoked,
                issuer_role: RealmOrganizationIssuerRole::GovernanceService,
            },
        ],
    }
}

fn mock_principal_control(realm_id: &str) -> OrgPrincipalControlPanel {
    OrgPrincipalControlPanel {
        realm_id: realm_id.to_string(),
        rows: vec![
            OrgPrincipalControl {
                organization_id: "did:web:acme.example".to_string(),
                controller: Some("did:web:acme.example#controller".to_string()),
                governance_service: Some("did:web:gov.acme.example".to_string()),
                issuer_role: RealmOrganizationIssuerRole::GovernanceService,
                account_authority_delegation_ref: Some(
                    "did:web:acme.example/delegations/gov-2026".to_string(),
                ),
                covered_control_scopes: vec![
                    RealmOrganizationControlScope::OfficialBadge,
                    RealmOrganizationControlScope::RealmAdmin,
                ],
                pcr_bootstrap_source: Some("threshold_quorum:acme-founders".to_string()),
                executed_by: Some("did:web:acme.example#alice-admin".to_string()),
                issued_at: Some("2026-01-10T08:00:00Z".to_string()),
                expires_at: Some("2026-07-10T08:00:00Z".to_string()),
                lifecycle: DelegationLifecycle::ExpiringSoon,
            },
            OrgPrincipalControl {
                organization_id: "did:web:acme.example".to_string(),
                controller: Some("did:web:acme.example#controller".to_string()),
                governance_service: None,
                issuer_role: RealmOrganizationIssuerRole::AccountAuthority,
                account_authority_delegation_ref: Some(
                    "did:web:acme.example/delegations/aa-2025".to_string(),
                ),
                covered_control_scopes: vec![RealmOrganizationControlScope::DirectoryListing],
                pcr_bootstrap_source: Some("did_document:acme-root".to_string()),
                executed_by: Some("did:web:acme.example#bob-ops".to_string()),
                issued_at: Some("2025-06-01T08:00:00Z".to_string()),
                expires_at: Some("2026-06-01T08:00:00Z".to_string()),
                lifecycle: DelegationLifecycle::Revoked,
            },
        ],
    }
}
