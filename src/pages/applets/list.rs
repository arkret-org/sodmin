use dioxus::prelude::*;

use crate::api::applets;
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
use crate::types::RegisterAppletRequest;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn AppletList() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut show_register_dialog = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| None::<String>);
    let mut name = use_signal(String::new);
    let mut endpoint_url = use_signal(String::new);
    let mut namespace = use_signal(String::new);
    let mut register_loading = use_signal(|| false);

    let page_val = *page.read();

    let mut data =
        use_resource(move || async move { applets::list_applets(page_val, PAGE_SIZE).await });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("applets.title"),
                description: t("applets.subtitle"),
                Button {
                    variant: ButtonVariant::Default,
                    onclick: move |_| show_register_dialog.set(true),
                    {t("common.create")}
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("applets.id")} }
                                    TableHead { {t("applets.name")} }
                                    TableHead { {t("applets.type")} }
                                    TableHead { {t("applets.status")} }
                                    TableHead { {t("applets.endpoint_url")} }
                                    TableHead { {t("applets.enabled")} }
                                    TableHead { {t("applets.registered_at")} }
                                    TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("applets.no_applets")}
                                        }
                                    }
                                } else {
                                    for applet in resp.data.iter() {
                                        {
                                            let id = applet.id.clone();
                                            let name = applet.name.clone();
                                            let applet_type = applet.applet_type.clone().unwrap_or_else(|| "-".to_string());
                                            let status = applet.status.clone().unwrap_or_else(|| "-".to_string());
                                            let endpoint_url = applet.endpoint_url.clone().unwrap_or_else(|| "-".to_string());
                                            let is_enabled = applet.is_enabled;
                                            let registered_at = applet.registered_at.clone().unwrap_or_else(|| "-".to_string());

                                            let id_for_toggle = id.clone();
                                            let id_for_delete = id.clone();

                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell { "{name}" }
                                                    TableCell { "{applet_type}" }
                                                    TableCell {
                                                        Badge { variant: BadgeVariant::Secondary, "{status}" }
                                                    }
                                                    TableCell { class: "max-w-[200px] truncate".to_string(), "{endpoint_url}" }
                                                    TableCell {
                                                        if is_enabled {
                                                            Badge { variant: BadgeVariant::Success, {t("common.enabled")} }
                                                        } else {
                                                            Badge { variant: BadgeVariant::Default, {t("common.disabled")} }
                                                        }
                                                    }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{registered_at}" }
                                                    TableCell { class: "text-right".to_string(),
                                                        div { class: "flex items-center justify-end gap-1",
                                                            Button {
                                                                variant: ButtonVariant::Ghost,
                                                                size: ButtonSize::Sm,
                                                                disabled: is_enabled,
                                                                onclick: {
                                                                    let id = id_for_toggle.clone();
                                                                    move |_| {
                                                                        let id = id.clone();
                                                                        spawn(async move {
                                                                            match applets::enable_applet(&id).await {
                                                                                Ok(_) => {
                                                                                    show_toast(&t("applets.toast_enabled"), ToastVariant::Success);
                                                                                    data.restart();
                                                                                }
                                                                                Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                                                            }
                                                                        });
                                                                    }
                                                                },
                                                                {t("common.enable")}
                                                            }
                                                            Button {
                                                                variant: ButtonVariant::Ghost,
                                                                size: ButtonSize::Sm,
                                                                disabled: !is_enabled,
                                                                onclick: {
                                                                    let id = id_for_toggle.clone();
                                                                    move |_| {
                                                                        let id = id.clone();
                                                                        spawn(async move {
                                                                            match applets::disable_applet(&id).await {
                                                                                Ok(_) => {
                                                                                    show_toast(&t("applets.toast_disabled"), ToastVariant::Success);
                                                                                    data.restart();
                                                                                }
                                                                                Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                                                            }
                                                                        });
                                                                    }
                                                                },
                                                                {t("common.disable")}
                                                            }
                                                            Button {
                                                                variant: ButtonVariant::Ghost,
                                                                size: ButtonSize::Sm,
                                                                onclick: {
                                                                    let id = id_for_delete.clone();
                                                                    move |_| show_delete_dialog.set(Some(id.clone()))
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
                        total: resp.total_or_len(),
                        per_page: PAGE_SIZE,
                        on_page_change: move |p| page.set(p),
                    }
                },
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        errcode: e.body.as_ref().map(|b| b.errcode.clone()),
                        request_id: e.request_id.clone(),
                        retry_after_ms: e.retry_after_ms,
                        on_retry: move |_| data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }

        Modal {
            open: *show_register_dialog.read(),
            title: t("applets.register"),
            on_close: move |_| show_register_dialog.set(false),
            div { class: "space-y-3",
                div { class: "space-y-1",
                    Label { r#for: "applet-name".to_string(), {t("applets.name")} }
                    Input {
                        value: name.read().clone(),
                        oninput: move |evt: FormEvent| name.set(evt.value()),
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "applet-endpoint".to_string(), {t("applets.endpoint_url")} }
                    Input {
                        value: endpoint_url.read().clone(),
                        oninput: move |evt: FormEvent| endpoint_url.set(evt.value()),
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "applet-namespace".to_string(), {t("applets.namespace")} }
                    Input {
                        value: namespace.read().clone(),
                        oninput: move |evt: FormEvent| namespace.set(evt.value()),
                    }
                }
            }
            DialogActions {
                confirm_text: t("common.create"),
                cancel_text: t("common.cancel"),
                confirm_loading: *register_loading.read(),
                on_cancel: move |_| show_register_dialog.set(false),
                on_confirm: move |_| {
                    register_loading.set(true);
                    let req = RegisterAppletRequest {
                        name: name.read().clone(),
                        endpoint_url: endpoint_url.read().clone(),
                        namespace: if namespace.read().is_empty() { None } else { Some(namespace.read().clone()) },
                        ..Default::default()
                    };
                    spawn(async move {
                        match applets::register_applet(&req).await {
                            Ok(_) => {
                                show_toast(&t("applets.toast_registered"), ToastVariant::Success);
                                show_register_dialog.set(false);
                                data.restart();
                            }
                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                        }
                        register_loading.set(false);
                    });
                },
            }
        }

        ConfirmDialog {
            open: show_delete_dialog.read().is_some(),
            title: t("common.delete"),
            description: t("applets.delete_confirm"),
            confirm_text: t("common.delete"),
            destructive: true,
            on_confirm: move |_| {
                if let Some(id) = show_delete_dialog.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match applets::delete_applet(&id).await {
                            Ok(_) => {
                                show_toast(&t("applets.toast_deleted"), ToastVariant::Success);
                                data.restart();
                            }
                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                        }
                    });
                }
                show_delete_dialog.set(None);
            },
            on_cancel: move |_| show_delete_dialog.set(None),
        }
    }
}
