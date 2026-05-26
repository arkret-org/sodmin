use dioxus::prelude::*;

use crate::api::federation;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn FederationList() -> Element {
    let mut peer_page = use_signal(|| 1u64);
    let mut rule_page = use_signal(|| 1u64);
    let mut show_add_rule = use_signal(|| false);
    let mut show_delete_rule = use_signal(|| None::<String>);
    let mut new_rule_domain = use_signal(String::new);
    let mut add_loading = use_signal(|| false);

    let peer_page_val = *peer_page.read();
    let rule_page_val = *rule_page.read();

    let mut peers = use_resource(move || async move {
        federation::list_federation_peers(peer_page_val, PAGE_SIZE).await
    });

    let mut rules = use_resource(move || async move {
        federation::list_federation_allow_rules(rule_page_val, PAGE_SIZE).await
    });

    rsx! {
        div { class: "space-y-8",
            PageHeader {
                title: t("federation.title"),
                description: t("federation.subtitle"),
            }

            Card {
                CardHeader {
                    CardTitle { {t("federation.peers")} }
                }
                CardContent {
                    match &*peers.read() {
                        Some(Ok(data)) => rsx! {
                            div { class: "rounded-md border",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("federation.domain")} }
                                            TableHead { {t("federation.status")} }
                                            TableHead { {t("federation.trust_level")} }
                                            TableHead { {t("federation.last_successful_txn")} }
                                            TableHead { {t("federation.last_error")} }
                                        }
                                    }
                                    TableBody {
                                        if data.data.is_empty() {
                                            TableRow {
                                                TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                                    {t("federation.no_peers")}
                                                }
                                            }
                                        } else {
                                            for peer in data.data.iter() {
                                                {
                                                    let domain = peer.domain.clone();
                                                    let status = peer.status.clone().unwrap_or_else(|| "-".to_string());
                                                    let trust_level = peer.trust_level.clone().unwrap_or_else(|| "-".to_string());
                                                    let last_txn = peer.last_successful_txn.clone().unwrap_or_else(|| "-".to_string());
                                                    let last_error = peer.last_error.clone().unwrap_or_else(|| "-".to_string());

                                                    rsx! {
                                                        TableRow {
                                                            TableCell {
                                                                Link {
                                                                    to: Route::FederationShow { domain: urlencoding::encode(&domain).to_string() },
                                                                    class: "font-medium text-primary hover:underline",
                                                                    "{domain}"
                                                                }
                                                            }
                                                            TableCell {
                                                                Badge { variant: BadgeVariant::Secondary, "{status}" }
                                                            }
                                                            TableCell { "{trust_level}" }
                                                            TableCell { class: "text-muted-foreground".to_string(), "{last_txn}" }
                                                            TableCell { class: "max-w-[200px] truncate text-destructive".to_string(), "{last_error}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Pagination {
                                page: peer_page_val,
                                total: data.total,
                                per_page: PAGE_SIZE,
                                on_page_change: move |p| peer_page.set(p),
                            }
                        },
                        Some(Err(e)) => rsx! {
                            ErrorBanner {
                                message: e.message.clone(),
                                errcode: e.body.as_ref().map(|b| b.errcode.clone()),
                                request_id: e.request_id.clone(),
                                retry_after_ms: e.retry_after_ms,
                                on_retry: move |_| peers.restart(),
                            }
                        },
                        None => rsx! { PageSkeleton {} },
                    }
                }
            }

            Card {
                CardHeader {
                    div { class: "flex items-center justify-between",
                        CardTitle { {t("federation.allow_rules")} }
                        Button {
                            variant: ButtonVariant::Default,
                            onclick: move |_| show_add_rule.set(true),
                            {t("federation.add_rule")}
                        }
                    }
                }
                CardContent {
                    match &*rules.read() {
                        Some(Ok(data)) => rsx! {
                            div { class: "rounded-md border",
                                Table {
                                    TableHeader {
                                        TableRow {
                                            TableHead { {t("federation.id")} }
                                            TableHead { {t("federation.domain")} }
                                            TableHead { {t("federation.rule_type")} }
                                            TableHead { {t("federation.created_at")} }
                                            TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                        }
                                    }
                                    TableBody {
                                        if data.data.is_empty() {
                                            TableRow {
                                                TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                                    {t("federation.no_rules")}
                                                }
                                            }
                                        } else {
                                            for rule in data.data.iter() {
                                                {
                                                    let id = rule.id.clone();
                                                    let domain = rule.domain.clone();
                                                    let rule_type = rule.rule_type.clone().unwrap_or_else(|| "-".to_string());
                                                    let created_at = rule.created_at.clone().unwrap_or_else(|| "-".to_string());
                                                    let id_for_delete = id.clone();

                                                    rsx! {
                                                        TableRow {
                                                            TableCell { class: "font-medium".to_string(), "{id}" }
                                                            TableCell { "{domain}" }
                                                            TableCell { "{rule_type}" }
                                                            TableCell { class: "text-muted-foreground".to_string(), "{created_at}" }
                                                            TableCell { class: "text-right".to_string(),
                                                                Button {
                                                                    variant: ButtonVariant::Ghost,
                                                                    onclick: {
                                                                        let id = id_for_delete.clone();
                                                                        move |_| show_delete_rule.set(Some(id.clone()))
                                                                    },
                                                                    {t("common.delete")}
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
                            Pagination {
                                page: rule_page_val,
                                total: data.total,
                                per_page: PAGE_SIZE,
                                on_page_change: move |p| rule_page.set(p),
                            }
                        },
                        Some(Err(e)) => rsx! {
                            ErrorBanner {
                                message: e.message.clone(),
                                errcode: e.body.as_ref().map(|b| b.errcode.clone()),
                                request_id: e.request_id.clone(),
                                retry_after_ms: e.retry_after_ms,
                                on_retry: move |_| rules.restart(),
                            }
                        },
                        None => rsx! { PageSkeleton {} },
                    }
                }
            }
        }

        if *show_add_rule.read() {
            div { class: "fixed inset-0 z-50 flex items-center justify-center",
                    div { class: "fixed inset-0 bg-black/80", onclick: move |_| show_add_rule.set(false) }
                    div { class: "relative z-50 w-full max-w-md rounded-lg border glass-panel p-6 shadow-lg space-y-4",
                        h2 { class: "text-lg font-semibold", {t("federation.add_rule")} }
                        div { class: "space-y-1",
                            Label { r#for: "rule-domain".to_string(), {t("federation.domain")} }
                            Input {
                                value: new_rule_domain.read().clone(),
                                oninput: move |evt: FormEvent| new_rule_domain.set(evt.value()),
                            }
                        }
                        div { class: "flex justify-end gap-2",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| show_add_rule.set(false),
                                {t("common.cancel")}
                            }
                            Button {
                                variant: ButtonVariant::Default,
                                disabled: *add_loading.read(),
                                onclick: move |_| {
                                    add_loading.set(true);
                                    let domain = new_rule_domain.read().clone();
                                    spawn(async move {
                                        match federation::add_federation_allow_rule(&domain).await {
                                            Ok(_) => {
                                                show_toast("Rule added", ToastVariant::Success);
                                                show_add_rule.set(false);
                                                new_rule_domain.set(String::new());
                                                rules.restart();
                                            }
                                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                        }
                                        add_loading.set(false);
                                    });
                                },
                                {t("common.create")}
                            }
                        }
                    }
                }
            }

        ConfirmDialog {
            open: show_delete_rule.read().is_some(),
            title: t("common.delete"),
            description: "Are you sure you want to delete this allow rule?".to_string(),
            confirm_text: t("common.delete"),
            destructive: true,
            on_confirm: move |_| {
                if let Some(id) = show_delete_rule.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match federation::delete_federation_allow_rule(&id).await {
                            Ok(_) => {
                                show_toast("Rule deleted", ToastVariant::Success);
                                rules.restart();
                            }
                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                        }
                    });
                }
                show_delete_rule.set(None);
            },
            on_cancel: move |_| show_delete_rule.set(None),
        }
    }
}
