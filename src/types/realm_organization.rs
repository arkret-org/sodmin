//! View DTOs for the Realm organization control surface (SOD-ORG-01..03).
//!
//! These structures are operator-view projections built **on top of** the
//! canonical SDK types — they never re-define protocol semantics. The verified
//! relationship rows reuse [`RealmOrganizationPayload`] and its enums verbatim
//! (`cokret_core::models`); coauth principal-control rows reuse the same
//! issuer-role / scope vocabulary plus coauth delegation metadata.
//!
//! Real data is not yet available:
//!   * SOD-ORG-01 data depends on soland's verified Realm organization
//!     projection (SOL-ORG-06).
//!   * SOD-ORG-02 data depends on coauth's organization-principal admin API
//!     (COA-ORG-05).
//! Until those land, the `api::realm_organization` layer returns mock rows
//! constructed from these DTOs (see the TODO markers there).

use serde::{Deserialize, Serialize};

pub use cokret_core::models::{
    RealmOrganizationControlScope, RealmOrganizationIssuerRole, RealmOrganizationRelationship,
};

/// Lifecycle bucket the operator view assigns to a verified relationship row.
///
/// soland's verified projection (SOL-ORG-06) MUST keep `revoked` / `expired` /
/// `stale` separate from `active` so the admin never mistakes a dead
/// relationship for live consent. We mirror that split here instead of
/// collapsing everything into `RealmOrganizationStatus` (which only encodes
/// the on-wire active/revoked discriminator).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationshipLifecycle {
    /// `status == active`, inside its validity window, not superseded.
    Active,
    /// Organization issued an explicit `status == revoked` statement.
    Revoked,
    /// `expires_at` is in the past (organization consent lapsed).
    Expired,
    /// Superseded by a newer statement or the captured Realm frontier no longer
    /// matches — present for completeness, never counted as verified.
    Stale,
}

impl RelationshipLifecycle {
    /// Only `Active` counts as a live, verified relationship.
    pub fn is_verified(self) -> bool {
        matches!(self, RelationshipLifecycle::Active)
    }
}

/// One verified Realm ↔ organization relationship row (SOD-ORG-01).
///
/// Field set mirrors [`RealmOrganizationPayload`] but only carries the
/// audit-relevant projection (no proof bytes — the operator surface does not
/// re-verify signatures; soland's projection already did).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifiedOrgRelationship {
    pub organization_id: String,
    pub relationship: RealmOrganizationRelationship,
    pub control_scopes: Vec<RealmOrganizationControlScope>,
    pub issued_at: String,
    pub not_before: Option<String>,
    pub expires_at: Option<String>,
    pub statement_id: String,
    pub supersedes_statement_id: Option<String>,
    pub revokes_statement_id: Option<String>,
    /// Lifecycle bucket assigned by soland's verified projection.
    pub lifecycle: RelationshipLifecycle,
    /// Issuer role of the authorization proof (organization DID vs delegated
    /// governance service / account authority / threshold quorum).
    pub issuer_role: RealmOrganizationIssuerRole,
}

/// SOD-ORG-01 panel payload: declared-only set + verified rows.
///
/// `declared_owning_organizations` is the Realm's self-declared
/// `owning_organizations` (`cokret_core::models::Realm.owning_organizations`),
/// which carries **no** organization consent on its own. The panel diffs it
/// against `verified` so the operator can see "who claims the organization" vs
/// "did the organization actually agree".
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealmOrganizationPanel {
    pub realm_id: String,
    /// Self-declared organizations from the Realm object (no consent implied).
    pub declared_owning_organizations: Vec<String>,
    /// Verified relationship rows from soland's projection (SOL-ORG-06).
    pub verified: Vec<VerifiedOrgRelationship>,
}

impl RealmOrganizationPanel {
    /// Declared organization DIDs that have **no** active verified relationship
    /// — i.e. claimed but unconfirmed by the organization.
    pub fn declared_without_verified(&self) -> Vec<String> {
        self.declared_owning_organizations
            .iter()
            .filter(|did| {
                !self
                    .verified
                    .iter()
                    .any(|r| &&r.organization_id == did && r.lifecycle.is_verified())
            })
            .cloned()
            .collect()
    }

    /// Verified organization DIDs that the Realm did **not** self-declare —
    /// consent exists but the Realm object omits the organization.
    pub fn verified_without_declared(&self) -> Vec<String> {
        self.verified
            .iter()
            .filter(|r| r.lifecycle.is_verified())
            .map(|r| r.organization_id.clone())
            .filter(|did| !self.declared_owning_organizations.contains(did))
            .collect()
    }
}

/// Lifecycle bucket for a coauth principal-control delegation (SOD-ORG-02).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DelegationLifecycle {
    Live,
    ExpiringSoon,
    Expired,
    Revoked,
}

/// One organization-principal control / delegation audit row (SOD-ORG-02).
///
/// Reuses [`RealmOrganizationIssuerRole`] for the delegated role vocabulary and
/// the same control-scope enum as the relationship rows so the operator reads a
/// single consistent scope language across both panels. The remaining fields
/// (controller / governance service / account-authority delegation / PCR
/// bootstrap source / `executed_by`) come from coauth COA-ORG-05.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrgPrincipalControl {
    pub organization_id: String,
    /// DID currently controlling the organization principal (controller of the
    /// organization DID document).
    pub controller: Option<String>,
    /// Governance service DID, when control is mediated by one.
    pub governance_service: Option<String>,
    /// Delegated role this row represents.
    pub issuer_role: RealmOrganizationIssuerRole,
    /// Account Authority delegation reference (DID-document delegation URL /
    /// policy object), when present.
    pub account_authority_delegation_ref: Option<String>,
    /// Control scopes the delegation authorizes.
    pub covered_control_scopes: Vec<RealmOrganizationControlScope>,
    /// PCR (Principal Control Root) bootstrap source — how the organization
    /// principal's control was originally rooted.
    pub pcr_bootstrap_source: Option<String>,
    /// Human admin / service principal that executed the decision. NOT the
    /// organization principal and NOT a shared organization account — only an
    /// executor identity for audit attribution.
    pub executed_by: Option<String>,
    pub issued_at: Option<String>,
    pub expires_at: Option<String>,
    pub lifecycle: DelegationLifecycle,
}

/// SOD-ORG-02 panel payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrgPrincipalControlPanel {
    pub realm_id: String,
    pub rows: Vec<OrgPrincipalControl>,
}
