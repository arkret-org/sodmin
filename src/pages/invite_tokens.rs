use dioxus::prelude::*;

use crate::api::invite_tokens;
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
use crate::types::CreateInviteTokenRequest;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn InviteTokenList() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut show_create_dialog = use_signal(|| false);
    let mut show_delete_dialog = use_signal(|| None::<String>);
    let mut uses_allowed = use_signal(String::new);
    let mut expires_at = use_signal(String::new);
    let mut realm_id = use_signal(String::new);
    let mut create_loading = use_signal(|| false);

    let page_val = *page.read();

    let mut data = use_resource(move || async move {
        invite_tokens::list_invite_tokens(page_val, PAGE_SIZE).await
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("invite_tokens.title"),
                description: t("invite_tokens.subtitle"),
                Button {
                    variant: ButtonVariant::Default,
                    onclick: move |_| show_create_dialog.set(true),
                    {t("common.create")}
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("invite_tokens.id")} }
                                    TableHead { {t("invite_tokens.token")} }
                                    TableHead { {t("invite_tokens.uses_allowed")} }
                                    TableHead { {t("invite_tokens.uses_completed")} }
                                    TableHead { {t("invite_tokens.uses_pending")} }
                                    TableHead { {t("invite_tokens.expires_at")} }
                                    TableHead { {t("invite_tokens.realm_id")} }
                                    TableHead { {t("invite_tokens.created_at")} }
                                    TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("invite_tokens.no_tokens")}
                                        }
                                    }
                                } else {
                                    for token in resp.data.iter() {
                                        {
                                            let id = token.id.clone();
                                            let tok = token.token.clone();
                                            let ua = token.uses_allowed.map(|v| v.to_string()).unwrap_or_else(|| t("invite_tokens.unlimited"));
                                            let uc = token.uses_completed.to_string();
                                            let up = token.uses_pending.to_string();
                                            let exp = token.expires_at.clone().unwrap_or_else(|| "-".to_string());
                                            let rid = token.realm_id.clone().unwrap_or_else(|| "-".to_string());
                                            let created = token.created_at.clone().unwrap_or_else(|| "-".to_string());

                                            let id_for_delete = id.clone();
                                            let tok_for_copy = tok.clone();

                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell {
                                                        div { class: "flex items-center gap-1 max-w-[200px]",
                                                            span { class: "font-mono text-xs truncate", "{tok}" }
                                                            Button {
                                                                variant: ButtonVariant::Ghost,
                                                                size: ButtonSize::Sm,
                                                                onclick: {
                                                                    let tok = tok_for_copy.clone();
                                                                    move |_| {
                                                                        // Use the typed Clipboard API instead of
                                                                        // string-interpolated `eval` — no injection
                                                                        // surface, and the toast only fires once the
                                                                        // write actually resolves.
                                                                        let tok = tok.clone();
                                                                        spawn(async move {
                                                                            let clipboard = web_sys::window().map(|w| w.navigator().clipboard());
                                                                            match clipboard {
                                                                                Some(clipboard) => {
                                                                                    match wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&tok)).await {
                                                                                        Ok(_) => show_toast(&t("invite_tokens.toast_copied"), ToastVariant::Success),
                                                                                        Err(_) => show_toast(&t("invite_tokens.toast_copy_failed"), ToastVariant::Error),
                                                                                    }
                                                                                }
                                                                                None => show_toast(&t("invite_tokens.toast_copy_failed"), ToastVariant::Error),
                                                                            }
                                                                        });
                                                                    }
                                                                },
                                                                {t("common.copy")}
                                                            }
                                                        }
                                                    }
                                                    TableCell { "{ua}" }
                                                    TableCell { "{uc}" }
                                                    TableCell { "{up}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{exp}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{rid}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{created}" }
                                                    TableCell { class: "text-right".to_string(),
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
                        on_retry: move |_| data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }

        Modal {
            open: *show_create_dialog.read(),
            title: t("invite_tokens.create"),
            on_close: move |_| show_create_dialog.set(false),
            div { class: "space-y-3",
                div { class: "space-y-1",
                    Label { r#for: "it-uses".to_string(), {t("invite_tokens.uses_allowed")} }
                    Input {
                        r#type: "number".to_string(),
                        value: uses_allowed.read().clone(),
                        oninput: move |evt: FormEvent| uses_allowed.set(evt.value()),
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "it-expires".to_string(), {t("invite_tokens.expires_at")} }
                    Input {
                        r#type: "datetime-local".to_string(),
                        value: expires_at.read().clone(),
                        oninput: move |evt: FormEvent| expires_at.set(evt.value()),
                    }
                }
                div { class: "space-y-1",
                    Label { r#for: "it-realm".to_string(), {t("invite_tokens.realm_id")} }
                    Input {
                        value: realm_id.read().clone(),
                        oninput: move |evt: FormEvent| realm_id.set(evt.value()),
                    }
                }
            }
            DialogActions {
                confirm_text: t("common.create"),
                cancel_text: t("common.cancel"),
                confirm_loading: *create_loading.read(),
                on_cancel: move |_| show_create_dialog.set(false),
                on_confirm: move |_| {
                    create_loading.set(true);
                    let req = CreateInviteTokenRequest {
                        uses_allowed: uses_allowed.read().parse().ok(),
                        expires_at: if expires_at.read().is_empty() { None } else { Some(expires_at.read().clone()) },
                        realm_id: if realm_id.read().is_empty() { None } else { Some(realm_id.read().clone()) },
                    };
                    spawn(async move {
                        match invite_tokens::create_invite_token(&req).await {
                            Ok(_) => {
                                show_toast(&t("invite_tokens.toast_created"), ToastVariant::Success);
                                show_create_dialog.set(false);
                                data.restart();
                            }
                            Err(e) => show_toast(&format!("{}: {}", t("common.failed"), e.message), ToastVariant::Error),
                        }
                        create_loading.set(false);
                    });
                },
            }
        }

        ConfirmDialog {
            open: show_delete_dialog.read().is_some(),
            title: t("common.delete"),
            description: t("invite_tokens.delete_confirm"),
            confirm_text: t("common.delete"),
            destructive: true,
            on_confirm: move |_| {
                if let Some(id) = show_delete_dialog.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match invite_tokens::delete_invite_token(&id).await {
                            Ok(_) => {
                                show_toast(&t("invite_tokens.toast_deleted"), ToastVariant::Success);
                                data.restart();
                            }
                            Err(e) => show_toast(&format!("{}: {}", t("common.failed"), e.message), ToastVariant::Error),
                        }
                    });
                }
                show_delete_dialog.set(None);
            },
            on_cancel: move |_| show_delete_dialog.set(None),
        }
    }
}
