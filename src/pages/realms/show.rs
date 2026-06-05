use chrono::Utc;
use dioxus::prelude::*;

use crate::api::realms;
use crate::components::realm_classification_badge::RealmClassificationBadge;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::components::ui::table::*;
use crate::router::Route;
use crate::utils::i18n::t;
use crate::utils::security::handle::display_sigil;
use crate::utils::security::primary_handle::{
    PrimaryHandleSelectInput, select_primary_handle_string,
};

#[component]
pub fn RealmShow(realm_id: String) -> Element {
    let realm_id_data = realm_id.clone();
    let realm_id_members = realm_id.clone();
    let mut data = use_resource(move || {
        let id = realm_id_data.clone();
        async move { realms::get_realm(&id).await }
    });
    let members = use_resource(move || {
        let id = realm_id_members.clone();
        async move { realms::list_realm_members(&id).await }
    });

    let realm_id_for_delete = realm_id.clone();

    rsx! {
        div { class: "space-y-6",
            match &*data.read() {
                Some(Ok(realm)) => {
                    let breadcrumbs = vec![
                        BreadcrumbItem { label: t("nav.realms"), route: Some(Route::RealmList {}) },
                        BreadcrumbItem { label: realm.title.as_deref().unwrap_or(&realm.id).to_string(), route: None },
                    ];
                    rsx! {
                        Breadcrumbs {
                            items: breadcrumbs,
                        }

                        PageHeader {
                            title: realm.title.as_deref().unwrap_or(&realm.id).to_string(),
                            // P3A.6 — Principal Control vs Collaboration
                            // Realm badge. Hidden when the field is absent
                            // on older soland deployments.
                            if let Some(ref cls) = realm.realm_class {
                                RealmClassificationBadge { realm_class: cls.clone() }
                            }
                        }

                        div { class: "grid gap-6 md:grid-cols-2",
                            Card {
                                CardHeader { CardTitle { {t("realms.overview")} } }
                                CardContent {
                                    div { class: "space-y-3",
                                    {field_row(t("realms.id"), realm.id.clone())}
                                    {field_row(t("realms.type"), realm_type_label(&realm))}
                                    {field_row(t("realms.discoverability"), realm.discoverability.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("realms.creator"), realm.created_by.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("realms.members"), realm.member_count.to_string())}
                                    {field_row(t("realms.encrypted"), if realm.is_encrypted { t("common.yes") } else { t("common.no") })}
                                    {field_row(t("realms.join_rule"), realm.join_rule.as_deref().unwrap_or("-").to_string())}
                                    {field_row(t("realms.status"), if realm.is_blocked { t("realms.blocked") } else { t("realms.active") })}
                                    {field_row(t("realms.created_at"), realm.created_at.as_deref().unwrap_or("-").to_string())}
                                        if let Some(ref topic) = realm.topic {
                                            div { class: "pt-2",
                                                p { class: "text-sm text-muted-foreground mb-1", {t("realms.topic")} }
                                                p { class: "text-sm", "{topic}" }
                                            }
                                        }
                                    }
                                }
                            }

                            Card {
                                CardHeader { CardTitle { {t("realms.actions")} } }
                                CardContent {
                                    div { class: "space-y-2",
                                        Button {
                                            variant: ButtonVariant::Destructive,
                                            class: "w-full".to_string(),
                                            onclick: move |_| {
                                                let rid = realm_id_for_delete.clone();
                                                spawn(async move {
                                                    let _ = realms::delete_realm(&rid).await;
                                                });
                                            },
                                            {t("realms.delete")}
                                        }
                                    }
                                }
                            }
                        }

                        Card {
                            CardHeader {
                                div { class: "flex items-start justify-between gap-2",
                                    div { class: "space-y-1",
                                        CardTitle { {t("realms.members")} }
                                        // R3.1 (MID-1) — admin rows now
                                        // source handle / display_name from
                                        // the effective MemberIdentity, not
                                        // the raw sync `members[]` fields.
                                        CardDescription { {t("realms.members_subtitle")} }
                                    }
                                    // R3.1 (MID-3) — quick jump to the
                                    // per-Realm identity audit diagnostic
                                    // page.
                                    Link {
                                        to: Route::RealmIdentityAudit { realm_id: realm.id.clone() },
                                        class: "text-xs text-primary hover:underline",
                                        {t("realms.open_identity_audit")}
                                    }
                                }
                            }
                            CardContent {
                                match &*members.read() {
                                    Some(Ok(member_list)) => {
                                        // R3.1 (ROST-2) — reserve the
                                        // truncation affordance for the
                                        // cursor-shaped Realm roster.
                                        let members_limited = false;
                                        let next_cursor: Option<String> = None;
                                        let shown = member_list.len();
                                        rsx! {
                                            Table {
                                                TableHeader {
                                                    TableRow {
                                                        TableHead { {t("realms.actor_id")} }
                                                        TableHead { {t("realms.identity")} }
                                                        TableHead { {t("realms.membership")} }
                                                        TableHead { {t("realms.role")} }
                                                        TableHead { {t("realms.joined_at")} }
                                                    }
                                                }
                                                TableBody {
                                                    for member in member_list.iter() {
                                                        {
                                                            // R3.2 (UI-SOD-3) —
                                                            // MemberIdentity no longer
                                                            // carries a handle. Derive the
                                                            // display handle by running
                                                            // §3.2.1 primary-handle
                                                            // selection over the visible
                                                            // `handle_claims` set when the
                                                            // roster discloses `subject_id`;
                                                            // fall back to any SPA-derived
                                                            // value the projection-join
                                                            // pass already filled.
                                                            let derived_handle = member
                                                                .subject_id
                                                                .as_deref()
                                                                .zip(member.handle_claims.as_deref())
                                                                .and_then(|(subject_id, claims)| {
                                                                    let input = PrimaryHandleSelectInput {
                                                                        subject_id,
                                                                        context: None,
                                                                        claim_set_snapshot: claims,
                                                                        accepted_issuers: &[],
                                                                        holder_primary_handle_at_as_of: None,
                                                                        resolution_as_of: Utc::now(),
                                                                    };
                                                                    select_primary_handle_string(&input)
                                                                })
                                                                .or_else(|| member.primary_handle.clone());
                                                            // When the SPA hasn't joined
                                                            // identity events yet (no derived
                                                            // handle / display_name and at
                                                            // least one identity_event_id),
                                                            // surface the "pending
                                                            // decryption" placeholder rather
                                                            // than falling back to the raw
                                                            // DID.
                                                            let identity_pending =
                                                                !member.identity_event_ids.is_empty()
                                                                    && derived_handle.is_none()
                                                                    && member.display_name.is_none();
                                                            let handle_canon =
                                                                derived_handle.clone().unwrap_or_default();
                                                            let sigil = if handle_canon.is_empty() {
                                                                String::new()
                                                            } else {
                                                                display_sigil(&handle_canon)
                                                            };
                                                            let display = member
                                                                .display_name
                                                                .clone()
                                                                .unwrap_or_default();
                                                            let membership = member
                                                                .membership
                                                                .clone()
                                                                .unwrap_or_else(|| "join".to_string());
                                                            rsx! {
                                                                TableRow {
                                                                    TableCell { class: "font-mono text-xs".to_string(), "{member.actor_id}" }
                                                                    TableCell {
                                                                        if identity_pending {
                                                                            Badge { variant: BadgeVariant::Outline,
                                                                                {t("realms.identity_pending")}
                                                                            }
                                                                        } else if !handle_canon.is_empty() || !display.is_empty() {
                                                                            div { class: "flex flex-col",
                                                                                if !display.is_empty() {
                                                                                    span { class: "text-sm font-medium", "{display}" }
                                                                                }
                                                                                if !handle_canon.is_empty() {
                                                                                    span { class: "text-xs font-mono", "{handle_canon}" }
                                                                                    span { class: "text-muted-foreground/80 text-[10px]", "{sigil}" }
                                                                                }
                                                                            }
                                                                        } else {
                                                                            // No identity events
                                                                            // and no joined
                                                                            // projection -- e.g.
                                                                            // bare `invite` row.
                                                                            // Acceptable to
                                                                            // render an em-dash;
                                                                            // raw DID stays in
                                                                            // the actor_id col.
                                                                            span { class: "text-muted-foreground", "-" }
                                                                        }
                                                                    }
                                                                    TableCell { {membership} }
                                                                    TableCell { {member.role.as_deref().unwrap_or("member")} }
                                                                    TableCell { {member.joined_at.as_deref().unwrap_or("-")} }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            // R3.1 (ROST-2) — "showing N of
                                            // many" affordance + load-more.
                                            // Hidden when the page wasn't
                                            // truncated.
                                            if members_limited {
                                                div { class: "mt-3 flex items-center justify-between text-xs text-muted-foreground",
                                                    span {
                                                        {format!(
                                                            "{} {} {}",
                                                            t("realms.members_showing"),
                                                            shown,
                                                            t("realms.members_of_many"),
                                                        )}
                                                    }
                                                    if next_cursor.is_some() {
                                                        Button {
                                                            variant: ButtonVariant::Outline,
                                                            disabled: true,
                                                            {t("realms.members_load_more")}
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    _ => rsx! { p { class: "text-muted-foreground", {t("common.loading")} } },
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! { ErrorBanner { message: e.message.clone(), on_retry: move |_| data.restart() } },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}

fn realm_type_label(realm: &crate::types::Realm) -> String {
    realm
        .realm_class
        .as_deref()
        .unwrap_or("collaboration")
        .to_owned()
}

fn field_row(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex justify-between py-1.5 border-b border-border/50 last:border-0",
            span { class: "text-sm text-muted-foreground", "{label}" }
            span { class: "text-sm font-medium", "{value}" }
        }
    }
}
