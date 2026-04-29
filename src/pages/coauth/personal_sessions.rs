use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::input::{Input, Label};
use crate::components::ui::loading::PageSkeleton;
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

        if *show_create.read() {
            div { class: "fixed inset-0 z-50 flex items-center justify-center",
                    div { class: "fixed inset-0 bg-black/80", onclick: move |_| show_create.set(false) }
                    div { class: "relative z-50 w-full max-w-md rounded-lg border glass-panel p-6 shadow-lg space-y-4",
                        h2 { class: "text-lg font-semibold", {t("coauth.personal_sessions.create")} }
                        div { class: "space-y-1",
                            Label { r#for: "ps-name".to_string(), {t("coauth.personal_sessions.name")} }
                            Input {
                                value: new_name.read().clone(),
                                oninput: move |evt: FormEvent| new_name.set(evt.value()),
                            }
                        }
                        div { class: "flex justify-end gap-2",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| show_create.set(false),
                                {t("common.cancel")}
                            }
                            Button {
                                variant: ButtonVariant::Default,
                                disabled: *create_loading.read(),
                                onclick: move |_| {
                                    create_loading.set(true);
                                    let name = new_name.read().clone();
                                    spawn(async move {
                                        match coauth::create_personal_session(&name).await {
                                            Ok(_) => {
                                                show_toast("Session created", ToastVariant::Success);
                                                show_create.set(false);
                                                new_name.set(String::new());
                                                data.restart();
                                            }
                                            Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                        }
                                        create_loading.set(false);
                                    });
                                },
                                {t("common.create")}
                            }
                        }
                    }
                }
            }

        ConfirmDialog {
            open: show_revoke.read().is_some(),
            title: t("coauth.personal_sessions.revoke"),
            description: "Are you sure you want to revoke this token?".to_string(),
            confirm_text: t("coauth.personal_sessions.revoke"),
            destructive: true,
            on_confirm: move |_| {
                if let Some(id) = show_revoke.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match coauth::revoke_personal_session(&id).await {
                            Ok(_) => {
                                show_toast("Session revoked", ToastVariant::Success);
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
            description: "This will invalidate the current token and generate a new one.".to_string(),
            confirm_text: t("coauth.personal_sessions.regenerate"),
            destructive: true,
            on_confirm: move |_| {
                if let Some(id) = show_regenerate.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match coauth::regenerate_personal_session(&id).await {
                            Ok(_) => {
                                show_toast("Session regenerated", ToastVariant::Success);
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
    }
}
