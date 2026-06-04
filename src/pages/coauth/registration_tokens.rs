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
pub fn RegistrationTokensPage() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut show_create = use_signal(|| false);
    let mut show_revoke = use_signal(|| None::<String>);
    let mut uses_allowed = use_signal(String::new);
    let mut create_loading = use_signal(|| false);

    let page_val = *page.read();

    let mut data =
        use_resource(
            move || async move { coauth::list_registration_tokens(page_val, PAGE_SIZE).await },
        );

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("coauth.registration_tokens.title"),
                description: t("coauth.registration_tokens.subtitle"),
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
                                    TableHead { {t("coauth.registration_tokens.id")} }
                                    TableHead { {t("coauth.registration_tokens.token")} }
                                    TableHead { {t("coauth.registration_tokens.uses_allowed")} }
                                    TableHead { {t("coauth.registration_tokens.uses_completed")} }
                                    TableHead { {t("coauth.registration_tokens.uses_pending")} }
                                    TableHead { {t("coauth.registration_tokens.expires_at")} }
                                    TableHead { {t("coauth.registration_tokens.revoked")} }
                                    TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("coauth.registration_tokens.no_tokens")}
                                        }
                                    }
                                } else {
                                    for token in resp.data.iter() {
                                        {
                                            let id = token.id.clone();
                                            let tok = token.token.clone().unwrap_or_else(|| "-".to_string());
                                            let ua = token.uses_allowed.map(|v| v.to_string()).unwrap_or_else(|| "Unlimited".to_string());
                                            let uc = token.uses_completed.to_string();
                                            let up = token.uses_pending.to_string();
                                            let exp = token.expires_at.clone().unwrap_or_else(|| "-".to_string());
                                            let is_revoked = token.is_revoked;

                                            let id_for_revoke = id.clone();

                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell {
                                                        span { class: "font-mono text-xs", "{tok}" }
                                                    }
                                                    TableCell { "{ua}" }
                                                    TableCell { "{uc}" }
                                                    TableCell { "{up}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{exp}" }
                                                    TableCell {
                                                        if is_revoked {
                                                            Badge { variant: BadgeVariant::Destructive, {t("coauth.registration_tokens.revoked")} }
                                                        } else {
                                                            Badge { variant: BadgeVariant::Success, {t("coauth.registration_tokens.active")} }
                                                        }
                                                    }
                                                    TableCell { class: "text-right".to_string(),
                                                        Button {
                                                            variant: ButtonVariant::Ghost,
                                                            size: ButtonSize::Sm,
                                                            disabled: is_revoked,
                                                            onclick: {
                                                                let id = id_for_revoke.clone();
                                                                move |_| show_revoke.set(Some(id.clone()))
                                                            },
                                                            {t("coauth.registration_tokens.revoke")}
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
            title: t("coauth.registration_tokens.create"),
            on_close: move |_| show_create.set(false),
            div { class: "space-y-1",
                Label { r#for: "rt-uses".to_string(), {t("coauth.registration_tokens.uses_allowed")} }
                Input {
                    r#type: "number".to_string(),
                    value: uses_allowed.read().clone(),
                    oninput: move |evt: FormEvent| uses_allowed.set(evt.value()),
                }
            }
            DialogActions {
                confirm_text: t("common.create"),
                cancel_text: t("common.cancel"),
                confirm_loading: *create_loading.read(),
                on_cancel: move |_| show_create.set(false),
                on_confirm: move |_| {
                    create_loading.set(true);
                    let ua: Option<u64> = uses_allowed.read().parse().ok();
                    spawn(async move {
                        match coauth::create_registration_token(ua).await {
                            Ok(_) => {
                                show_toast("Token created", ToastVariant::Success);
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
            open: show_revoke.read().is_some(),
            title: t("coauth.registration_tokens.revoke"),
            description: "Are you sure you want to revoke this token?".to_string(),
            confirm_text: t("coauth.registration_tokens.revoke"),
            destructive: true,
            on_confirm: move |_| {
                if let Some(id) = show_revoke.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match coauth::revoke_registration_token(&id).await {
                            Ok(_) => {
                                show_toast("Token revoked", ToastVariant::Success);
                                data.restart();
                            }
                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                        }
                    });
                }
                show_revoke.set(None);
            },
            on_cancel: move |_| show_revoke.set(None),
        }
    }
}
