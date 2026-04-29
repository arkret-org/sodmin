use dioxus::prelude::*;

use crate::api::capabilities;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::GrantCapabilityRequest;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn CapabilityList() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut show_grant_dialog = use_signal(|| false);
    let mut show_revoke_dialog = use_signal(|| None::<String>);
    let mut grantee_id = use_signal(String::new);
    let mut capability_name = use_signal(String::new);
    let mut scope = use_signal(String::new);
    let mut expires_at = use_signal(String::new);
    let mut grant_loading = use_signal(|| false);

    let page_val = *page.read();

    let mut data = use_resource(move || async move {
        capabilities::list_capabilities(page_val, PAGE_SIZE).await
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("capabilities.title"),
                description: t("capabilities.subtitle"),
                Button {
                    variant: ButtonVariant::Default,
                    onclick: move |_| show_grant_dialog.set(true),
                    {t("capabilities.grant")}
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("capabilities.id")} }
                                    TableHead { {t("capabilities.grantor_id")} }
                                    TableHead { {t("capabilities.grantee_id")} }
                                    TableHead { {t("capabilities.capability")} }
                                    TableHead { {t("capabilities.scope")} }
                                    TableHead { {t("capabilities.granted_at")} }
                                    TableHead { {t("capabilities.expires_at")} }
                                    TableHead { {t("capabilities.revoked")} }
                                    TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("capabilities.no_capabilities")}
                                        }
                                    }
                                } else {
                                    for cap in resp.data.iter() {
                                        {
                                            let id = cap.id.clone();
                                            let grantor_id = cap.grantor_id.clone();
                                            let grantee = cap.grantee_id.clone();
                                            let cap_name = cap.capability.clone();
                                            let cap_scope = cap.scope.clone().unwrap_or_else(|| "-".to_string());
                                            let granted_at = cap.granted_at.clone().unwrap_or_else(|| "-".to_string());
                                            let expires = cap.expires_at.clone().unwrap_or_else(|| "-".to_string());
                                            let is_revoked = cap.is_revoked;

                                            let id_for_revoke = id.clone();

                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{grantor_id}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{grantee}" }
                                                    TableCell { "{cap_name}" }
                                                    TableCell { class: "max-w-[200px] truncate".to_string(), "{cap_scope}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{granted_at}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{expires}" }
                                                    TableCell {
                                                        if is_revoked {
                                                            Badge { variant: BadgeVariant::Destructive, {t("capabilities.revoked")} }
                                                        } else {
                                                            Badge { variant: BadgeVariant::Success, {t("capabilities.active")} }
                                                        }
                                                    }
                                                    TableCell { class: "text-right".to_string(),
                                                        Button {
                                                            variant: ButtonVariant::Ghost,
                                                            size: ButtonSize::Sm,
                                                            disabled: is_revoked,
                                                            onclick: {
                                                                let id = id_for_revoke.clone();
                                                                move |_| show_revoke_dialog.set(Some(id.clone()))
                                                            },
                                                            {t("capabilities.revoke")}
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

        if *show_grant_dialog.read() {
            div { class: "fixed inset-0 z-50 flex items-center justify-center",
                    div { class: "fixed inset-0 bg-black/80", onclick: move |_| show_grant_dialog.set(false) }
                    div { class: "relative z-50 w-full max-w-md rounded-lg border glass-panel p-6 shadow-lg space-y-4",
                        h2 { class: "text-lg font-semibold", {t("capabilities.grant")} }
                        div { class: "space-y-3",
                            div { class: "space-y-1",
                                Label { r#for: "cap-grantee".to_string(), {t("capabilities.grantee_id")} }
                                Input {
                                    value: grantee_id.read().clone(),
                                    oninput: move |evt: FormEvent| grantee_id.set(evt.value()),
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "cap-name".to_string(), {t("capabilities.capability")} }
                                Input {
                                    value: capability_name.read().clone(),
                                    oninput: move |evt: FormEvent| capability_name.set(evt.value()),
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "cap-scope".to_string(), {t("capabilities.scope")} }
                                Input {
                                    value: scope.read().clone(),
                                    oninput: move |evt: FormEvent| scope.set(evt.value()),
                                }
                            }
                            div { class: "space-y-1",
                                Label { r#for: "cap-expires".to_string(), {t("capabilities.expires_at")} }
                                Input {
                                    r#type: "datetime-local".to_string(),
                                    value: expires_at.read().clone(),
                                    oninput: move |evt: FormEvent| expires_at.set(evt.value()),
                                }
                            }
                        }
                        div { class: "flex justify-end gap-2",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| show_grant_dialog.set(false),
                                {t("common.cancel")}
                            }
                            Button {
                                variant: ButtonVariant::Default,
                                disabled: *grant_loading.read(),
                                onclick: move |_| {
                                    grant_loading.set(true);
                                    let req = GrantCapabilityRequest {
                                        grantee_id: grantee_id.read().clone(),
                                        capability: capability_name.read().clone(),
                                        scope: if scope.read().is_empty() { None } else { Some(scope.read().clone()) },
                                        expires_at: if expires_at.read().is_empty() { None } else { Some(expires_at.read().clone()) },
                                        ..Default::default()
                                    };
                                    spawn(async move {
                                        match capabilities::grant_capability(&req).await {
                                            Ok(_) => {
                                                show_toast("Capability granted", ToastVariant::Success);
                                                show_grant_dialog.set(false);
                                                data.restart();
                                            }
                                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                        }
                                        grant_loading.set(false);
                                    });
                                },
                                {t("capabilities.grant")}
                            }
                        }
                    }
                }
            }

        ConfirmDialog {
            open: show_revoke_dialog.read().is_some(),
            title: t("capabilities.revoke"),
            description: "Are you sure you want to revoke this capability?".to_string(),
            confirm_text: t("capabilities.revoke"),
            destructive: true,
            on_confirm: move |_| {
                if let Some(id) = show_revoke_dialog.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match capabilities::revoke_capability(&id).await {
                            Ok(_) => {
                                show_toast("Capability revoked", ToastVariant::Success);
                                data.restart();
                            }
                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                        }
                    });
                }
                show_revoke_dialog.set(None);
            },
            on_cancel: move |_| show_revoke_dialog.set(None),
        }
    }
}
