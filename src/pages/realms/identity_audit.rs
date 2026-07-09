//! Realm identity audit diagnostic page.
//!
//! Read-only operator view that lists, per actor in a Realm, the
//! current effective `ck.member.identity.update` event ids and the
//! `member_display_state_digest` projection the SPA computed locally.
//! Since arkret-spec @ b56cab1 `MemberIdentity` no longer carries a
//! handle, so the handle column is derived by running §3.2.1
//! primary-handle selection over the row's visible handle-claim set; a
//! "handle changed since" hint surfaces when the captured
//! `handle_at_time` differs from the current primary. Useful for
//! triaging cross-actor identity drift without hitting the soland audit
//! log directly.
//!
//! Route: `/realms/:realm_id/identity-audit`.
//!
//! Data plumbing is not yet wired through soland's admin API — the
//! page renders a stub-state today and ships the full live view once
//! the SDK `MemberIdentity` decrypt pipeline (inkson MID-4) lands.
// TODO: hook `api::realms::list_realm_identity_audit(realm_id)` once
// soland exposes the admin endpoint that aggregates the effective
// identity projection per actor.

use chrono::Utc;
use dioxus::prelude::*;

use crate::components::handle_change_hint::HandleChangeHint;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::components::ui::table::*;
use crate::router::Route;
use crate::types::RealmIdentityAuditRow;
use crate::utils::i18n::t;
use crate::utils::security::handle::display_sigil;
use crate::utils::security::primary_handle::{
    PrimaryHandleSelectInput, SubjectRender, render_subject, select_primary_handle_string,
};

#[component]
pub fn IdentityAuditPage(realm_id: String) -> Element {
    // TODO: replace with a `use_resource` against the soland admin
    // endpoint. The stub rows below are intentionally empty so the
    // operator sees the "data pending" affordance rather than a fake
    // result set.
    let rows: Vec<RealmIdentityAuditRow> = Vec::new();

    let breadcrumbs = vec![
        BreadcrumbItem {
            label: t("nav.spaces"),
            route: Some(Route::SpaceList {}),
        },
        BreadcrumbItem {
            label: realm_id.clone(),
            route: Some(Route::RealmShow {
                realm_id: realm_id.clone(),
            }),
        },
        BreadcrumbItem {
            label: t("realm_identity_audit.title"),
            route: None,
        },
    ];

    rsx! {
        div { class: "space-y-6",
            Breadcrumbs { items: breadcrumbs }

            PageHeader {
                title: t("realm_identity_audit.title"),
                description: t("realm_identity_audit.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    {t("common.refresh")}
                }
            }

            // R3.1 stub banner — surfaces the TODO(R4) state to the
            // operator instead of pretending to be empty data.
            div {
                class: "rounded-md border border-amber-600 bg-amber-600/10 px-3 py-2 text-sm space-y-1",
                role: "alert",
                p { class: "font-semibold text-amber-700 dark:text-amber-200",
                    span { class: "mr-2", "\u{2139}" }
                    {t("realm_identity_audit.stub_title")}
                }
                p { class: "text-xs text-amber-700/90 dark:text-amber-200/90",
                    {t("realm_identity_audit.stub_body")}
                }
            }

            Card {
                CardHeader {
                    CardTitle { {t("realm_identity_audit.actors_title")} }
                    CardDescription { {t("realm_identity_audit.actors_subtitle")} }
                }
                CardContent {
                    Table {
                        TableHeader {
                            TableRow {
                                TableHead { {t("realm_identity_audit.col_actor")} }
                                TableHead { {t("realm_identity_audit.col_primary_handle")} }
                                TableHead { {t("realm_identity_audit.col_display_name")} }
                                TableHead { {t("realm_identity_audit.col_event_ids")} }
                                TableHead { {t("realm_identity_audit.col_state_digest")} }
                                TableHead { {t("realm_identity_audit.col_cache")} }
                            }
                        }
                        TableBody {
                            if rows.is_empty() {
                                TableRow {
                                    TableCell {
                                        class: "text-center text-muted-foreground py-6".to_string(),
                                        colspan: 99,
                                        {t("realm_identity_audit.empty")}
                                    }
                                }
                            } else {
                                for row in rows.iter() {
                                    {
                                        let actor = row.actor_id.clone();
                                        // R3.2 (UI-SOD-3) — MemberIdentity no
                                        // longer carries a handle. Derive the
                                        // display handle by running §3.2.1
                                        // selection over the visible claim set
                                        // when `subject_id` is disclosed,
                                        // degrading through display name then a
                                        // truncated DID (§3.8.2 fallback ladder).
                                        let subject_id =
                                            row.subject_id.clone().unwrap_or_else(|| row.actor_id.clone());
                                        let claims = row.handle_claims.clone().unwrap_or_default();
                                        let sel = PrimaryHandleSelectInput {
                                            subject_id: &subject_id,
                                            context: None,
                                            claim_set_snapshot: &claims,
                                            accepted_issuers: &[],
                                            holder_primary_handle_at_as_of: None,
                                            resolution_as_of: Utc::now(),
                                        };
                                        let handle_canon = select_primary_handle_string(&sel)
                                            .or_else(|| row.primary_handle.clone());
                                        let rendered =
                                            render_subject(&sel, row.display_name.as_deref());
                                        let rendered_label = match &rendered {
                                            SubjectRender::Verified(h) => h.clone(),
                                            SubjectRender::NameOnly(n) => n.clone(),
                                            SubjectRender::Unresolved(d) => d.clone(),
                                        };
                                        let degraded = !matches!(rendered, SubjectRender::Verified(_));
                                        // R3.2 (UI-SOD-5) — captured handle at
                                        // projection time, for the change hint.
                                        let handle_at_time = row.handle_at_time.clone();
                                        let current_primary = handle_canon.clone();
                                        let sigil = handle_canon
                                            .as_deref()
                                            .map(display_sigil)
                                            .unwrap_or_default();
                                        let display = row.display_name.clone().unwrap_or_else(|| "-".to_string());
                                        let event_ids = if row.identity_event_ids.is_empty() {
                                            "-".to_string()
                                        } else {
                                            row.identity_event_ids.join(", ")
                                        };
                                        let digest = row
                                            .member_display_state_digest
                                            .clone()
                                            .unwrap_or_else(|| "-".to_string());
                                        let drift = row.cache_drift;
                                        rsx! {
                                            TableRow {
                                                TableCell { class: "font-mono text-xs".to_string(), "{actor}" }
                                                TableCell { class: "font-mono text-xs".to_string(),
                                                    div { class: "flex flex-col",
                                                        div { class: "flex items-center gap-1",
                                                            span {
                                                                class: if degraded { "text-muted-foreground" } else { "" },
                                                                "{rendered_label}"
                                                            }
                                                            HandleChangeHint {
                                                                handle_at_time,
                                                                current_primary,
                                                            }
                                                        }
                                                        if !sigil.is_empty() {
                                                            span { class: "text-muted-foreground/80 text-[10px]", "{sigil}" }
                                                        }
                                                    }
                                                }
                                                TableCell { class: "text-xs".to_string(), "{display}" }
                                                TableCell { class: "font-mono text-xs max-w-[280px] truncate".to_string(), "{event_ids}" }
                                                TableCell { class: "font-mono text-xs max-w-[280px] truncate".to_string(), "{digest}" }
                                                TableCell {
                                                    if drift {
                                                        Badge { variant: BadgeVariant::Destructive,
                                                            {t("realm_identity_audit.cache_drift")}
                                                        }
                                                    } else {
                                                        Badge { variant: BadgeVariant::Success,
                                                            {t("realm_identity_audit.cache_ok")}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
