//! Realm delivery-binding-policy read-only operations view.
//!
//! Renders the effective `delivery_binding_policy` cell for a single
//! Realm (the security boundary). This view is **read-only** in sodmin:
//! mutating `allowed_recipient_services` / `binding_source_policy` is a
//! general-management action that strands through events / inkson, not the
//! operations console. `policy_frontier` is reducer-owned.
//!
//! Below the policy summary the page lists each Realm member with their
//! effective `member_delivery_binding.recipient_service_id` and a
//! routability check against the allowed list, plus the handover panel.

use dioxus::prelude::*;

use crate::api::delivery_binding;
use crate::components::delivery_binding_handover_panel::DeliveryBindingHandoverPanel;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::utils::i18n::t;

#[component]
pub fn DeliveryBindingPage(realm_id: String) -> Element {
    if is_placeholder_resource_id(&realm_id) {
        return selection_required_state("Realm");
    }

    let id_for_policy = realm_id.clone();
    let id_for_members = realm_id.clone();
    let id_for_handovers = realm_id.clone();

    let mut policy_data = use_resource(move || {
        let id = id_for_policy.clone();
        async move { delivery_binding::get_delivery_binding_policy(&id).await }
    });
    let mut members_data = use_resource(move || {
        let id = id_for_members.clone();
        async move { delivery_binding::list_member_routability(&id).await }
    });
    let mut handovers_data = use_resource(move || {
        let id = id_for_handovers.clone();
        async move { delivery_binding::list_delivery_binding_handovers(&id).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("delivery_binding.title"),
                description: format!("{}: {}", t("delivery_binding.realm"), realm_id),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        policy_data.restart();
                        members_data.restart();
                        handovers_data.restart();
                    },
                    {t("common.refresh")}
                }
            }

            match &*policy_data.read() {
                Some(Ok(policy)) => {
                    let frontier = policy.policy_frontier.clone().unwrap_or_else(|| "-".to_string());
                    let updated_at = policy.updated_at.clone().unwrap_or_else(|| "-".to_string());
                    let binding_source = policy.binding_source_policy.clone().unwrap_or_else(|| "-".to_string());
                    let allowed_list = policy.allowed_recipient_services.clone();
                    rsx! {
                        Card {
                            CardHeader {
                                CardTitle { {t("delivery_binding.policy_title")} }
                                CardDescription { {t("delivery_binding.policy_subtitle")} }
                            }
                            CardContent {
                                div { class: "space-y-4",
                                    // Read-only operations view. Realm
                                    // delivery-binding policy writes strand
                                    // through events / inkson, not sodmin.
                                    div { class: "grid gap-3 sm:grid-cols-2",
                                        div {
                                            p { class: "text-xs text-muted-foreground", {t("delivery_binding.policy_frontier")} }
                                            p { class: "text-sm font-mono break-all", "{frontier}" }
                                        }
                                        div {
                                            p { class: "text-xs text-muted-foreground", {t("delivery_binding.updated_at")} }
                                            p { class: "text-sm font-mono", "{updated_at}" }
                                        }
                                    }

                                    div { class: "space-y-1",
                                        p { class: "text-xs text-muted-foreground", {t("delivery_binding.binding_source_policy")} }
                                        p { class: "text-sm font-mono break-all", "{binding_source}" }
                                    }

                                    div { class: "space-y-2",
                                        p { class: "text-xs text-muted-foreground", {t("delivery_binding.allowed_recipient_services")} }
                                        if allowed_list.is_empty() {
                                            p { class: "text-sm text-muted-foreground", {t("delivery_binding.allowed_empty")} }
                                        } else {
                                            div { class: "flex flex-wrap gap-1",
                                                for did in allowed_list.iter() {
                                                    Badge {
                                                        variant: BadgeVariant::Secondary,
                                                        class: "font-mono text-xs".to_string(),
                                                        "{did}"
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
                Some(Err(e)) => rsx! {
                    ErrorBanner { message: e.message.clone(), on_retry: move |_| policy_data.restart() }
                },
                None => rsx! { PageSkeleton {} },
            }

            Card {
                CardHeader {
                    CardTitle { {t("delivery_binding.members_title")} }
                    CardDescription { {t("delivery_binding.members_subtitle")} }
                }
                CardContent {
                    match &*members_data.read() {
                        Some(Ok(resp)) => rsx! {
                            if resp.data.is_empty() {
                                p { class: "text-sm text-muted-foreground py-4 text-center",
                                    {t("delivery_binding.members_empty")}
                                }
                            } else {
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("delivery_binding.member_actor")} }
                                            TableHead { {t("delivery_binding.member_recipient")} }
                                            TableHead { {t("delivery_binding.member_status")} }
                                            TableHead { {t("delivery_binding.member_routability")} }
                                        }
                                    }
                                    TableBody {
                                        for row in resp.data.iter() {
                                            {
                                                let actor = if let Some(name) = row.display_name.as_ref() {
                                                    format!("{} ({})", name, row.actor_id)
                                                } else {
                                                    row.actor_id.clone()
                                                };
                                                let recipient = row.recipient_service_id.clone().unwrap_or_else(|| "-".to_string());
                                                let status = row.delivery_status.clone().unwrap_or_else(|| "-".to_string());
                                                let routable = row.in_allowed_list;
                                                rsx! {
                                                    TableRow {
                                                        key: "{row.actor_id}",
                                                        TableCell { class: "text-xs".to_string(), "{actor}" }
                                                        TableCell { class: "text-xs font-mono max-w-[260px] truncate".to_string(), "{recipient}" }
                                                        TableCell { class: "text-xs".to_string(), "{status}" }
                                                        TableCell {
                                                            if routable {
                                                                Badge { variant: BadgeVariant::Success, {t("delivery_binding.routable")} }
                                                            } else {
                                                                Badge { variant: BadgeVariant::Destructive, {t("delivery_binding.unroutable")} }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(e)) => rsx! {
                            ErrorBanner { message: e.message.clone(), on_retry: move |_| members_data.restart() }
                        },
                        None => rsx! { PageSkeleton {} },
                    }
                }
            }

            match &*handovers_data.read() {
                Some(Ok(resp)) => rsx! {
                    DeliveryBindingHandoverPanel { rows: resp.data.clone() }
                },
                Some(Err(e)) => rsx! {
                    ErrorBanner { message: e.message.clone(), on_retry: move |_| handovers_data.restart() }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}
