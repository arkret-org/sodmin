use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::modal::{DialogActions, Modal};
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn UpstreamProvidersPage() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut show_create = use_signal(|| false);
    let mut show_delete = use_signal(|| None::<String>);
    let mut issuer = use_signal(String::new);
    let mut client_id = use_signal(String::new);
    let mut create_loading = use_signal(|| false);

    let page_val = *page.read();

    let mut data =
        use_resource(
            move || async move { coauth::list_upstream_providers(page_val, PAGE_SIZE).await },
        );

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("coauth.upstream_providers.title"),
                description: t("coauth.upstream_providers.subtitle"),
                Button {
                    variant: ButtonVariant::Default,
                    onclick: move |_| show_create.set(true),
                    {t("common.create")}
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("coauth.upstream_providers.id")} }
                                    TableHead { {t("coauth.upstream_providers.issuer")} }
                                    TableHead { {t("coauth.upstream_providers.client_id")} }
                                    TableHead { {t("coauth.upstream_providers.enabled")} }
                                    TableHead { {t("coauth.upstream_providers.created_at")} }
                                    TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("coauth.upstream_providers.no_providers")}
                                        }
                                    }
                                } else {
                                    for provider in resp.data.iter() {
                                        {
                                            let id = provider.id.clone();
                                            let p_issuer = provider.issuer.clone().unwrap_or_else(|| "-".to_string());
                                            let p_client_id = provider.client_id.clone().unwrap_or_else(|| "-".to_string());
                                            let is_enabled = provider.is_enabled;
                                            let created = provider.created_at.clone().unwrap_or_else(|| "-".to_string());

                                            let id_for_toggle = id.clone();
                                            let id_for_delete = id.clone();

                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell { class: "max-w-[200px] truncate".to_string(), "{p_issuer}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{p_client_id}" }
                                                    TableCell {
                                                        if is_enabled {
                                                            Badge { variant: BadgeVariant::Success, {t("common.enabled")} }
                                                        } else {
                                                            Badge { variant: BadgeVariant::Default, {t("common.disabled")} }
                                                        }
                                                    }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{created}" }
                                                    TableCell { class: "text-right".to_string(),
                                                        div { class: "flex items-center justify-end gap-1",
                                                            Button {
                                                                variant: ButtonVariant::Ghost,
                                                                size: ButtonSize::Sm,
                                                                onclick: {
                                                                    let id = id_for_toggle.clone();
                                                                    let new_state = !is_enabled;
                                                                    move |_| {
                                                                        let id = id.clone();
                                                                        spawn(async move {
                                                                            match coauth::toggle_upstream_provider(&id, new_state).await {
                                                                                Ok(_) => {
                                                                                    show_toast(if new_state { "Enabled" } else { "Disabled" }, ToastVariant::Success);
                                                                                    data.restart();
                                                                                }
                                                                                Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                                                            }
                                                                        });
                                                                    }
                                                                },
                                                                if is_enabled { {t("common.disable")} } else { {t("common.enable")} }
                                                            }
                                                            Button {
                                                                variant: ButtonVariant::Ghost,
                                                                size: ButtonSize::Sm,
                                                                onclick: {
                                                                    let id = id_for_delete.clone();
                                                                    move |_| show_delete.set(Some(id.clone()))
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
                    }

                    Pagination {
                        page: page_val,
                        total: resp.total,
                        per_page: PAGE_SIZE,
                        on_page_change: move |p| page.set(p),
                    }
                },
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        on_retry: move |_| data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }

        Modal {
            open: *show_create.read(),
            title: t("coauth.upstream_providers.create"),
            on_close: move |_| show_create.set(false),
            div { class: "space-y-3",
                div { class: "space-y-1",
                    Label { r#for: "up-issuer".to_string(), {t("coauth.upstream_providers.issuer")} }
                    Input {
                        value: issuer.read().clone(),
                        oninput: move |evt: FormEvent| issuer.set(evt.value()),
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "up-client".to_string(), {t("coauth.upstream_providers.client_id")} }
                    Input {
                        value: client_id.read().clone(),
                        oninput: move |evt: FormEvent| client_id.set(evt.value()),
                    }
                }
            }
            DialogActions {
                confirm_text: t("common.create"),
                cancel_text: t("common.cancel"),
                confirm_loading: *create_loading.read(),
                on_cancel: move |_| show_create.set(false),
                on_confirm: move |_| {
                    create_loading.set(true);
                    let body = serde_json::json!({
                        "issuer": issuer.read().clone(),
                        "client_id": client_id.read().clone(),
                    });
                    spawn(async move {
                        match coauth::create_upstream_provider(&body).await {
                            Ok(_) => {
                                show_toast("Provider created", ToastVariant::Success);
                                show_create.set(false);
                                data.restart();
                            }
                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                        }
                        create_loading.set(false);
                    });
                },
            }
        }

        ConfirmDialog {
            open: show_delete.read().is_some(),
            title: t("common.delete"),
            description: "Are you sure you want to delete this provider?".to_string(),
            confirm_text: t("common.delete"),
            destructive: true,
            on_confirm: move |_| {
                if let Some(id) = show_delete.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match coauth::delete_upstream_provider(&id).await {
                            Ok(_) => {
                                show_toast("Provider deleted", ToastVariant::Success);
                                data.restart();
                            }
                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                        }
                    });
                }
                show_delete.set(None);
            },
            on_cancel: move |_| show_delete.set(None),
        }
    }
}
