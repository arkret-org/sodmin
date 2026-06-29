use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::ui::button::{Button, ButtonVariant};
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
pub fn PersonalSessionsPage() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut show_create = use_signal(|| false);
    let mut show_revoke = use_signal(|| None::<String>);
    let mut show_regenerate = use_signal(|| None::<String>);
    let mut new_name = use_signal(String::new);
    let mut create_loading = use_signal(|| false);
    // One-shot access token captured from create/regenerate. The token is
    // only returned once by coauth; surface it in a copy-once modal and
    // drop it from memory when the modal closes (never persisted).
    let mut revealed_token = use_signal(|| None::<String>);

    let page_val = *page.read();

    let mut data =
        use_resource(
            move || async move { coauth::list_personal_sessions(page_val, PAGE_SIZE).await },
        );

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("coauth.personal_sessions.title"),
                description: t("coauth.personal_sessions.subtitle"),
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
                                    TableHead { {t("coauth.personal_sessions.id")} }
                                    TableHead { {t("coauth.personal_sessions.user_id")} }
                                    TableHead { {t("coauth.personal_sessions.name")} }
                                    TableHead { {t("coauth.personal_sessions.created_at")} }
                                    TableHead { {t("coauth.personal_sessions.last_active")} }
                                    TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("coauth.personal_sessions.no_sessions")}
                                        }
                                    }
                                } else {
                                    for session in resp.data.iter() {
                                        {
                                            let id = session.id.clone();
                                            let user_id = session.user_id.clone().unwrap_or_else(|| "-".to_string());
                                            let name = session.name.clone().unwrap_or_else(|| "-".to_string());
                                            let created = session.created_at.clone().unwrap_or_else(|| "-".to_string());
                                            let last_active = session.last_active_at.clone().unwrap_or_else(|| "-".to_string());

                                            let id_for_revoke = id.clone();
                                            let id_for_regen = id.clone();

                                            rsx! {
                                                TableRow {
                                                    key: "{id}",
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{user_id}" }
                                                    TableCell { "{name}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{created}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{last_active}" }
                                                    TableCell { class: "text-right".to_string(),
                                                        div { class: "flex items-center justify-end gap-1",
                                                            Button {
                                                                variant: ButtonVariant::Ghost,
                                                                onclick: {
                                                                    let id = id_for_regen.clone();
                                                                    move |_| show_regenerate.set(Some(id.clone()))
                                                                },
                                                                {t("coauth.personal_sessions.regenerate")}
                                                            }
                                                            Button {
                                                                variant: ButtonVariant::Ghost,
                                                                onclick: {
                                                                    let id = id_for_revoke.clone();
                                                                    move |_| show_revoke.set(Some(id.clone()))
                                                                },
                                                                {t("coauth.personal_sessions.revoke")}
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
            title: t("coauth.personal_sessions.create"),
            on_close: move |_| show_create.set(false),
            div { class: "space-y-1",
                Label { r#for: "ps-name".to_string(), {t("coauth.personal_sessions.name")} }
                Input {
                    value: new_name.read().clone(),
                    oninput: move |evt: FormEvent| new_name.set(evt.value()),
                }
            }
            DialogActions {
                confirm_text: t("common.create"),
                cancel_text: t("common.cancel"),
                confirm_loading: *create_loading.read(),
                on_cancel: move |_| show_create.set(false),
                on_confirm: move |_| {
                    create_loading.set(true);
                    let name = new_name.read().clone();
                    spawn(async move {
                        match coauth::create_personal_session(&name).await {
                            Ok(oneshot) => {
                                show_create.set(false);
                                new_name.set(String::new());
                                match oneshot.access_token {
                                    Some(tok) if !tok.is_empty() => {
                                        revealed_token.set(Some(tok));
                                    }
                                    _ => show_toast(&t("coauth.personal_sessions.toast_created"), ToastVariant::Success),
                                }
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
            title: t("coauth.personal_sessions.revoke"),
            description: t("coauth.personal_sessions.revoke_confirm"),
            confirm_text: t("coauth.personal_sessions.revoke"),
            destructive: true,
            on_confirm: move |_| {
                if let Some(id) = show_revoke.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match coauth::revoke_personal_session(&id).await {
                            Ok(_) => {
                                show_toast(&t("coauth.personal_sessions.toast_revoked"), ToastVariant::Success);
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

        ConfirmDialog {
            open: show_regenerate.read().is_some(),
            title: t("coauth.personal_sessions.regenerate"),
            description: t("coauth.personal_sessions.regenerate_confirm"),
            confirm_text: t("coauth.personal_sessions.regenerate"),
            destructive: true,
            on_confirm: move |_| {
                if let Some(id) = show_regenerate.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match coauth::regenerate_personal_session(&id).await {
                            Ok(oneshot) => {
                                match oneshot.access_token {
                                    Some(tok) if !tok.is_empty() => {
                                        revealed_token.set(Some(tok));
                                    }
                                    _ => show_toast(&t("coauth.personal_sessions.toast_regenerated"), ToastVariant::Success),
                                }
                                data.restart();
                            }
                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                        }
                    });
                }
                show_regenerate.set(None);
            },
            on_cancel: move |_| show_regenerate.set(None),
        }

        // One-shot token reveal. coauth returns the access token exactly
        // once on create/regenerate; show it with a copy button and a
        // "won't be shown again" warning, then drop it on close.
        {
            let token = revealed_token.read().clone();
            rsx! {
                Modal {
                    open: token.is_some(),
                    title: t("coauth.personal_sessions.token_reveal_title"),
                    on_close: move |_| revealed_token.set(None),
                    div { class: "space-y-3",
                        p { class: "text-sm text-amber-700 dark:text-amber-300",
                            {t("coauth.personal_sessions.token_reveal_warning")}
                        }
                        if let Some(tok) = token.clone() {
                            div { class: "rounded-md border bg-muted/40 px-3 py-2 font-mono text-xs break-all",
                                "{tok}"
                            }
                            Button {
                                variant: ButtonVariant::Default,
                                onclick: move |_| {
                                    let tok = tok.clone();
                                    spawn(async move {
                                        if let Some(clipboard) = web_sys::window().map(|w| w.navigator().clipboard()) {
                                            let _ = wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&tok)).await;
                                            show_toast(&t("coauth.personal_sessions.token_copied"), ToastVariant::Success);
                                        }
                                    });
                                },
                                {t("common.copy")}
                            }
                        }
                    }
                    DialogActions {
                        confirm_text: t("common.close"),
                        cancel_text: t("common.close"),
                        on_cancel: move |_| revealed_token.set(None),
                        on_confirm: move |_| revealed_token.set(None),
                    }
                }
            }
        }
    }
}
