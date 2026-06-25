//! Realm organization control admin page (SOD-ORG-01..03).
//!
//! Read-only operator surface that answers two questions an admin must be able
//! to answer at a glance:
//!   * **SOD-ORG-01** — "Who claims to own/govern this Realm, and did the
//!     organization actually consent?" Shows the verified relationship rows
//!     from soland's projection next to the declared-only `owning_organizations`
//!     set, diffing the two. `revoked` / `expired` / `stale` rows are bucketed
//!     separately and never counted as verified.
//!   * **SOD-ORG-02** — "Who controls the organization principal, and through
//!     what proof?" Shows controller / governance service / Account-Authority
//!     delegation / PCR bootstrap source and the `executed_by` executor (which
//!     is explicitly NOT the organization principal / a shared account).
//!     Expiring / revoked delegations are highlighted.
//!   * **SOD-ORG-03** — security operation entry points (revoke / renew
//!     delegation, revoke realm relationship). These are skeleton-only: each
//!     button shows the impact scope and is wired to a TODO that MUST call the
//!     coauth / soland standard authorization API — sodmin never edits the DB
//!     or emits product-private events directly.
//!
//! Route: `/realms/:realm_id/organization`.
//!
//! Data plumbing is mock-backed today (see `api::realm_organization`): the
//! verified relationship data depends on soland SOL-ORG-06 and the principal
//! control data depends on coauth COA-ORG-05.

use dioxus::prelude::*;

use crate::api::realm_organization;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::components::ui::table::*;
use crate::router::Route;
use crate::types::{
    DelegationLifecycle, OrgPrincipalControl, RealmOrganizationControlScope,
    RealmOrganizationIssuerRole, RealmOrganizationPanel, RealmOrganizationRelationship,
    RelationshipLifecycle, VerifiedOrgRelationship,
};
use crate::utils::i18n::t;

#[component]
pub fn RealmOrganization(realm_id: String) -> Element {
    if is_placeholder_resource_id(&realm_id) {
        return selection_required_state("Realm");
    }

    let realm_id_panel = realm_id.clone();
    let mut panel = use_resource(move || {
        let id = realm_id_panel.clone();
        async move { realm_organization::get_realm_organization_panel(&id).await }
    });

    let realm_id_control = realm_id.clone();
    let mut control = use_resource(move || {
        let id = realm_id_control.clone();
        async move { realm_organization::get_org_principal_control_panel(&id).await }
    });

    let breadcrumbs = vec![
        BreadcrumbItem {
            label: t("nav.realms"),
            route: Some(Route::RealmList {}),
        },
        BreadcrumbItem {
            label: realm_id.clone(),
            route: Some(Route::RealmShow {
                realm_id: realm_id.clone(),
            }),
        },
        BreadcrumbItem {
            label: t("realm_organization.title"),
            route: None,
        },
    ];

    rsx! {
        div { class: "space-y-6",
            Breadcrumbs { items: breadcrumbs }

            PageHeader {
                title: t("realm_organization.title"),
                description: t("realm_organization.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        panel.restart();
                        control.restart();
                    },
                    {t("common.refresh")}
                }
            }

            // Mock-data banner — surfaces SOL-ORG-06 / COA-ORG-05 dependency so
            // the operator never mistakes the fixture for live data.
            div {
                class: "rounded-md border border-amber-600 bg-amber-600/10 px-3 py-2 text-sm space-y-1",
                role: "alert",
                p { class: "font-semibold text-amber-700 dark:text-amber-200",
                    span { class: "mr-2", "\u{2139}" }
                    {t("realm_organization.stub_title")}
                }
                p { class: "text-xs text-amber-700/90 dark:text-amber-200/90",
                    {t("realm_organization.stub_body")}
                }
            }

            // ── SOD-ORG-01 — verified relationship panel ──
            match &*panel.read() {
                Some(Ok(p)) => rsx! { VerifiedRelationshipCard { panel: p.clone() } },
                Some(Err(e)) => rsx! {
                    ErrorBanner { message: e.message.clone(), on_retry: move |_| panel.restart() }
                },
                None => rsx! { PageSkeleton {} },
            }

            // ── SOD-ORG-02 — organization principal control audit ──
            match &*control.read() {
                Some(Ok(c)) => rsx! { PrincipalControlCard { rows: c.rows.clone() } },
                Some(Err(e)) => rsx! {
                    ErrorBanner { message: e.message.clone(), on_retry: move |_| control.restart() }
                },
                None => rsx! { PageSkeleton {} },
            }

            // ── SOD-ORG-03 — security operations (skeleton) ──
            SecurityOperationsCard {}
        }
    }
}

// ── SOD-ORG-01 ──

#[component]
fn VerifiedRelationshipCard(panel: RealmOrganizationPanel) -> Element {
    let declared_only = panel.declared_without_verified();
    let verified_undeclared = panel.verified_without_declared();

    rsx! {
        Card {
            CardHeader {
                CardTitle { {t("realm_organization.verified_title")} }
                CardDescription { {t("realm_organization.verified_subtitle")} }
            }
            CardContent {
                div { class: "space-y-4",
                    // Declared-only diff: claimed but no organization consent.
                    if !declared_only.is_empty() {
                        div {
                            class: "rounded-md border border-amber-600/60 bg-amber-600/5 px-3 py-2 text-sm",
                            p { class: "font-semibold mb-1", {t("realm_organization.declared_only_title")} }
                            ul { class: "list-disc pl-5 space-y-0.5",
                                for did in declared_only.iter() {
                                    li { class: "font-mono text-xs", "{did}" }
                                }
                            }
                        }
                    }
                    // Verified but not self-declared: consent exists, Realm
                    // object omits the org.
                    if !verified_undeclared.is_empty() {
                        div {
                            class: "rounded-md border border-sky-600/60 bg-sky-600/5 px-3 py-2 text-sm",
                            p { class: "font-semibold mb-1", {t("realm_organization.verified_undeclared_title")} }
                            ul { class: "list-disc pl-5 space-y-0.5",
                                for did in verified_undeclared.iter() {
                                    li { class: "font-mono text-xs", "{did}" }
                                }
                            }
                        }
                    }

                    Table {
                        TableHeader {
                            TableRow {
                                TableHead { {t("realm_organization.col_org")} }
                                TableHead { {t("realm_organization.col_relationship")} }
                                TableHead { {t("realm_organization.col_status")} }
                                TableHead { {t("realm_organization.col_scopes")} }
                                TableHead { {t("realm_organization.col_issued")} }
                                TableHead { {t("realm_organization.col_expires")} }
                                TableHead { {t("realm_organization.col_statement")} }
                            }
                        }
                        TableBody {
                            if panel.verified.is_empty() {
                                TableRow {
                                    TableCell {
                                        class: "text-center text-muted-foreground py-6".to_string(),
                                        colspan: 99,
                                        {t("realm_organization.verified_empty")}
                                    }
                                }
                            } else {
                                for row in panel.verified.iter() {
                                    {relationship_row(row)}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn relationship_row(row: &VerifiedOrgRelationship) -> Element {
    let scopes = row
        .control_scopes
        .iter()
        .map(|s| scope_label(*s))
        .collect::<Vec<_>>()
        .join(", ");
    rsx! {
        TableRow {
            TableCell { class: "font-mono text-xs".to_string(), "{row.organization_id}" }
            TableCell { class: "text-xs".to_string(), {relationship_label(row.relationship)} }
            TableCell { {lifecycle_badge(row.lifecycle)} }
            TableCell { class: "text-xs max-w-[260px]".to_string(), "{scopes}" }
            TableCell { class: "text-xs".to_string(), "{row.issued_at}" }
            TableCell { class: "text-xs".to_string(),
                {row.expires_at.clone().unwrap_or_else(|| t("realm_organization.no_expiry"))}
            }
            TableCell { class: "font-mono text-xs".to_string(),
                div { class: "flex flex-col",
                    span { "{row.statement_id}" }
                    span { class: "text-muted-foreground/80 text-[10px]",
                        {issuer_role_label(row.issuer_role)}
                    }
                }
            }
        }
    }
}

/// Lifecycle badge — `revoked` / `expired` / `stale` are visually distinct from
/// `active` so they are never read as verified consent.
fn lifecycle_badge(lc: RelationshipLifecycle) -> Element {
    let (variant, key) = match lc {
        RelationshipLifecycle::Active => (BadgeVariant::Success, "realm_organization.lifecycle_active"),
        RelationshipLifecycle::Revoked => {
            (BadgeVariant::Destructive, "realm_organization.lifecycle_revoked")
        }
        RelationshipLifecycle::Expired => {
            (BadgeVariant::Destructive, "realm_organization.lifecycle_expired")
        }
        RelationshipLifecycle::Stale => (BadgeVariant::Secondary, "realm_organization.lifecycle_stale"),
    };
    rsx! { Badge { variant, {t(key)} } }
}

// ── SOD-ORG-02 ──

#[component]
fn PrincipalControlCard(rows: Vec<OrgPrincipalControl>) -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { {t("realm_organization.control_title")} }
                CardDescription { {t("realm_organization.control_subtitle")} }
            }
            CardContent {
                Table {
                    TableHeader {
                        TableRow {
                            TableHead { {t("realm_organization.col_org")} }
                            TableHead { {t("realm_organization.col_controller")} }
                            TableHead { {t("realm_organization.col_governance")} }
                            TableHead { {t("realm_organization.col_role")} }
                            TableHead { {t("realm_organization.col_delegation")} }
                            TableHead { {t("realm_organization.col_pcr")} }
                            TableHead { {t("realm_organization.col_executed_by")} }
                            TableHead { {t("realm_organization.col_expires")} }
                            TableHead { {t("realm_organization.col_delegation_status")} }
                        }
                    }
                    TableBody {
                        if rows.is_empty() {
                            TableRow {
                                TableCell {
                                    class: "text-center text-muted-foreground py-6".to_string(),
                                    colspan: 99,
                                    {t("realm_organization.control_empty")}
                                }
                            }
                        } else {
                            for row in rows.iter() {
                                {principal_control_row(row)}
                            }
                        }
                    }
                }
            }
        }
    }
}

fn principal_control_row(row: &OrgPrincipalControl) -> Element {
    let dash = "-".to_string();
    let scopes = row
        .covered_control_scopes
        .iter()
        .map(|s| scope_label(*s))
        .collect::<Vec<_>>()
        .join(", ");
    rsx! {
        TableRow {
            TableCell { class: "font-mono text-xs".to_string(), "{row.organization_id}" }
            TableCell { class: "font-mono text-xs".to_string(),
                {row.controller.clone().unwrap_or_else(|| dash.clone())}
            }
            TableCell { class: "font-mono text-xs".to_string(),
                {row.governance_service.clone().unwrap_or_else(|| dash.clone())}
            }
            TableCell { class: "text-xs".to_string(), {issuer_role_label(row.issuer_role)} }
            TableCell { class: "font-mono text-xs max-w-[220px] truncate".to_string(),
                div { class: "flex flex-col",
                    span { {row.account_authority_delegation_ref.clone().unwrap_or_else(|| dash.clone())} }
                    if !scopes.is_empty() {
                        span { class: "text-muted-foreground/80 text-[10px]", "{scopes}" }
                    }
                }
            }
            TableCell { class: "font-mono text-xs".to_string(),
                {row.pcr_bootstrap_source.clone().unwrap_or_else(|| dash.clone())}
            }
            // `executed_by` — executor identity only, NOT the organization
            // principal and NOT a shared account.
            TableCell { class: "font-mono text-xs".to_string(),
                div { class: "flex flex-col",
                    span { {row.executed_by.clone().unwrap_or_else(|| dash.clone())} }
                    if row.executed_by.is_some() {
                        span { class: "text-muted-foreground/80 text-[10px]",
                            {t("realm_organization.executor_note")}
                        }
                    }
                }
            }
            TableCell { class: "text-xs".to_string(),
                {row.expires_at.clone().unwrap_or_else(|| t("realm_organization.no_expiry"))}
            }
            TableCell { {delegation_badge(row.lifecycle)} }
        }
    }
}

/// Highlight expiring-soon / revoked delegations.
fn delegation_badge(lc: DelegationLifecycle) -> Element {
    let (variant, key) = match lc {
        DelegationLifecycle::Live => (BadgeVariant::Success, "realm_organization.delegation_live"),
        DelegationLifecycle::ExpiringSoon => {
            (BadgeVariant::Outline, "realm_organization.delegation_expiring")
        }
        DelegationLifecycle::Expired => {
            (BadgeVariant::Destructive, "realm_organization.delegation_expired")
        }
        DelegationLifecycle::Revoked => {
            (BadgeVariant::Destructive, "realm_organization.delegation_revoked")
        }
    };
    rsx! { Badge { variant, {t(key)} } }
}

// ── SOD-ORG-03 — security operations skeleton ──

#[component]
fn SecurityOperationsCard() -> Element {
    rsx! {
        Card {
            CardHeader {
                CardTitle { {t("realm_organization.ops_title")} }
                CardDescription { {t("realm_organization.ops_subtitle")} }
            }
            CardContent {
                div { class: "space-y-4",
                    // Impact-scope preview shown before any operation. These are
                    // the boundaries a revoke/renew touches: official badge,
                    // governance policy, directory listing, durability/delivery
                    // binding policy.
                    div {
                        class: "rounded-md border border-border bg-muted/30 px-3 py-2 text-sm",
                        p { class: "font-semibold mb-1", {t("realm_organization.impact_title")} }
                        ul { class: "list-disc pl-5 space-y-0.5 text-xs",
                            li { {t("realm_organization.impact_official_badge")} }
                            li { {t("realm_organization.impact_governance_policy")} }
                            li { {t("realm_organization.impact_directory_listing")} }
                            li { {t("realm_organization.impact_binding_policy")} }
                        }
                    }

                    // TODO(SOD-ORG-03): each action MUST be routed through the
                    // coauth / soland standard authorization API so the emitted
                    // event verifies as a standard `ck.realm.organization`
                    // flow (cotest acceptance). sodmin MUST NOT mutate the DB or
                    // emit product-private events directly. Buttons are disabled
                    // until those endpoints exist (COA-ORG-05 / SOL-ORG-06).
                    div { class: "flex flex-wrap gap-2",
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: true,
                            {t("realm_organization.op_revoke_delegation")}
                        }
                        Button {
                            variant: ButtonVariant::Outline,
                            disabled: true,
                            {t("realm_organization.op_renew_delegation")}
                        }
                        Button {
                            variant: ButtonVariant::Destructive,
                            disabled: true,
                            {t("realm_organization.op_revoke_relationship")}
                        }
                    }
                    p { class: "text-xs text-muted-foreground",
                        {t("realm_organization.ops_pending_note")}
                    }
                }
            }
        }
    }
}

// ── label helpers (display only; never re-derive protocol semantics) ──

fn relationship_label(r: RealmOrganizationRelationship) -> String {
    let key = match r {
        RealmOrganizationRelationship::Owner => "realm_organization.rel_owner",
        RealmOrganizationRelationship::Governance => "realm_organization.rel_governance",
        RealmOrganizationRelationship::Sponsor => "realm_organization.rel_sponsor",
        RealmOrganizationRelationship::DirectoryCertifier => "realm_organization.rel_directory_certifier",
    };
    t(key)
}

fn issuer_role_label(r: RealmOrganizationIssuerRole) -> String {
    let key = match r {
        RealmOrganizationIssuerRole::OrganizationDid => "realm_organization.role_organization_did",
        RealmOrganizationIssuerRole::GovernanceService => "realm_organization.role_governance_service",
        RealmOrganizationIssuerRole::AccountAuthority => "realm_organization.role_account_authority",
        RealmOrganizationIssuerRole::ThresholdQuorum => "realm_organization.role_threshold_quorum",
    };
    t(key)
}

fn scope_label(s: RealmOrganizationControlScope) -> String {
    // Machine scope names are stable wire tokens; render them verbatim so the
    // operator sees the exact `control_scopes[]` value, not a lossy gloss.
    match s {
        RealmOrganizationControlScope::OfficialBadge => "official_badge",
        RealmOrganizationControlScope::RealmAdmin => "realm_admin",
        RealmOrganizationControlScope::NotaryControl => "notary_control",
        RealmOrganizationControlScope::PolicyServer => "policy_server",
        RealmOrganizationControlScope::DeliveryBindingPolicy => "delivery_binding_policy",
        RealmOrganizationControlScope::DurabilityPolicy => "durability_policy",
        RealmOrganizationControlScope::ModerationPolicy => "moderation_policy",
        RealmOrganizationControlScope::RetentionPolicy => "retention_policy",
        RealmOrganizationControlScope::DirectoryListing => "directory_listing",
        RealmOrganizationControlScope::PlaintextVisibleService => "plaintext_visible_service",
    }
    .to_string()
}
