//! T6.2 §3 — Realm delivery-binding-policy editor (post realm-rework).
//!
//! Renders the effective `delivery_binding_policy` cell for a single
//! Realm (the security boundary; pre realm-rework these were "Spaces")
//! and lets the operator mutate the editable fields
//! (`allowed_recipient_services`, `binding_source_policy`).
//! `policy_frontier` is reducer-owned and surfaced read-only.
//!
//! Below the editor the page lists each Realm member with their
//! effective `delivery_binding.recipient_service_did` and a routability
//! check against the allowed list.
//
// TODO(realm-rework): once the Realm link-graph visualisation lands,
// embed it under the routability table — for now a placeholder card
// is rendered.

use dioxus::prelude::*;

use crate::api::delivery_binding;
use crate::components::delivery_binding_handover_panel::DeliveryBindingHandoverPanel;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::UpdateDeliveryBindingPolicyRequest;
use crate::utils::i18n::t;

#[component]
pub fn DeliveryBindingPolicy(realm_id: String) -> Element {
    let id_for_policy = realm_id.clone();
    let id_for_members = realm_id.clone();
    let id_for_handovers = realm_id.clone();
    let id_for_save = realm_id.clone();

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

    let mut allowed_input = use_signal(String::new);
    let mut new_recipient = use_signal(String::new);
    let mut binding_source_policy = use_signal(String::new);
    let mut hydrated = use_signal(|| false);
    let mut saving = use_signal(|| false);

    // Hydrate the local editable state from the server response once
    // it arrives.
    if !*hydrated.read() {
        if let Some(Ok(policy)) = policy_data.read().as_ref() {
            allowed_input.set(policy.allowed_recipient_services.join(", "));
            binding_source_policy.set(policy.binding_source_policy.clone().unwrap_or_default());
            hydrated.set(true);
        }
    }

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("delivery_binding.title"),
                description: format!("{}: {}", t("delivery_binding.realm"), realm_id),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        hydrated.set(false);
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
                    let allowed_list = policy.allowed_recipient_services.clone();
                    rsx! {
                        Card {
                            CardHeader {
                                CardTitle { {t("delivery_binding.policy_title")} }
                                CardDescription { {t("delivery_binding.policy_subtitle")} }
                            }
                            CardContent {
                                div { class: "space-y-4",
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

                                    div { class: "space-y-2",
                                        Label {
                                            r#for: "binding-source-policy".to_string(),
                                            {t("delivery_binding.binding_source_policy")}
                                        }
                                        Input {
                                            value: binding_source_policy.read().clone(),
                                            oninput: move |evt: FormEvent| binding_source_policy.set(evt.value()),
                                        }
                                        p { class: "text-xs text-muted-foreground",
                                            {t("delivery_binding.binding_source_hint")}
                                        }
                                    }

                                    div { class: "space-y-2",
                                        Label { r#for: "allowed-recipient-services".to_string(),
                                            {t("delivery_binding.allowed_recipient_services")}
                                        }
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
                                        div { class: "flex gap-2",
                                            Input {
                                                value: new_recipient.read().clone(),
                                                oninput: move |evt: FormEvent| new_recipient.set(evt.value()),
                                            }
                                            Button {
                                                variant: ButtonVariant::Outline,
                                                size: ButtonSize::Sm,
                                                onclick: {
                                                    move |_| {
                                                        let v = new_recipient.read().trim().to_string();
                                                        if v.is_empty() { return; }
                                                        let mut current: Vec<String> = allowed_input
                                                            .read()
                                                            .split(',')
                                                            .map(|s| s.trim().to_string())
                                                            .filter(|s| !s.is_empty())
                                                            .collect();
                                                        if !current.iter().any(|c| c == &v) {
                                                            current.push(v);
                                                        }
                                                        allowed_input.set(current.join(", "));
                                                        new_recipient.set(String::new());
                                                    }
                                                },
                                                {t("delivery_binding.add_recipient")}
                                            }
                                        }
                                        Label { r#for: "allowed-edit".to_string(),
                                            {t("delivery_binding.allowed_edit_hint")}
                                        }
                                        Input {
                                            value: allowed_input.read().clone(),
                                            oninput: move |evt: FormEvent| allowed_input.set(evt.value()),
                                        }
                                    }

                                    div { class: "flex justify-end",
                                        Button {
                                            variant: ButtonVariant::Default,
                                            disabled: *saving.read(),
                                            onclick: {
                                                let realm_id = id_for_save.clone();
                                                move |_| {
                                                    let list: Vec<String> = allowed_input
                                                        .read()
                                                        .split(',')
                                                        .map(|s| s.trim().to_string())
                                                        .filter(|s| !s.is_empty())
                                                        .collect();
                                                    let bsp_raw = binding_source_policy.read().trim().to_string();
                                                    let req = UpdateDeliveryBindingPolicyRequest {
                                                        allowed_recipient_services: Some(list),
                                                        binding_source_policy: if bsp_raw.is_empty() { None } else { Some(bsp_raw) },
                                                    };
                                                    let realm_id = realm_id.clone();
                                                    saving.set(true);
                                                    spawn(async move {
                                                        match delivery_binding::update_delivery_binding_policy(&realm_id, &req).await {
                                                            Ok(_) => {
                                                                show_toast(&t("delivery_binding.save_ok"), ToastVariant::Success);
                                                                hydrated.set(false);
                                                                policy_data.restart();
                                                                members_data.restart();
                                                            }
                                                            Err(e) => show_toast(&format!("{}: {}", t("delivery_binding.save_fail"), e.message), ToastVariant::Error),
                                                        }
                                                        saving.set(false);
                                                    });
                                                }
                                            },
                                            {t("delivery_binding.save")}
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
                                                let recipient = row.recipient_service_did.clone().unwrap_or_else(|| "-".to_string());
                                                let status = row.delivery_status.clone().unwrap_or_else(|| "-".to_string());
                                                let routable = row.in_allowed_list;
                                                rsx! {
                                                    TableRow {
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

            // TODO(realm-rework): replace this placeholder with the real
            // Realm link-graph visualisation once the data surface ships.
            Card {
                CardHeader {
                    CardTitle { {t("delivery_binding.link_graph_title")} }
                    CardDescription { {t("delivery_binding.link_graph_subtitle")} }
                }
                CardContent {
                    p { class: "text-sm text-muted-foreground py-6 text-center",
                        {t("delivery_binding.link_graph_placeholder")}
                    }
                }
            }
        }
    }
}
