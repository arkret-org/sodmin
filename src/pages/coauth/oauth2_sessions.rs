use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::dangerous_action_dialog::{DangerousActionDialog, confirmation_suffix};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn OAuth2SessionsPage() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut pending_finish = use_signal(|| None::<String>);
    let mut in_flight = use_signal(|| None::<String>);

    let mut data = use_resource(move || async move {
        let page_val = *page.read();
        coauth::list_oauth2_sessions(page_val, PAGE_SIZE).await
    });
    let page_val = *page.read();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("coauth.oauth2_sessions.title"),
                description: t("coauth.oauth2_sessions.subtitle"),
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("coauth.oauth2_sessions.id")} }
                                    TableHead { {t("coauth.oauth2_sessions.user_id")} }
                                    TableHead { {t("coauth.oauth2_sessions.client_id")} }
                                    TableHead { {t("coauth.oauth2_sessions.scope")} }
                                    TableHead { {t("coauth.oauth2_sessions.human_name")} }
                                    TableHead { {t("coauth.oauth2_sessions.created_at")} }
                                    TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("coauth.oauth2_sessions.no_sessions")}
                                        }
                                    }
                                } else {
                                    for session in resp.data.iter() {
                                        {
                                            let id = session.id.clone();
                                            let user_id = session.user_id.clone().unwrap_or_else(|| "-".to_string());
                                            let client_id = session.client_id.clone().unwrap_or_else(|| "-".to_string());
                                            let scope = session.scope.clone().unwrap_or_else(|| "-".to_string());
                                            let human_name = session.human_name.clone().unwrap_or_else(|| "-".to_string());
                                            let created = session.created_at.clone().unwrap_or_else(|| "-".to_string());

                                            let id_for_finish = id.clone();
                                            let row_in_flight = in_flight
                                                .read()
                                                .as_deref()
                                                .map(|current| current == id.as_str())
                                                .unwrap_or(false);

                                            rsx! {
                                                TableRow {
                                                    key: "{id}",
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{user_id}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{client_id}" }
                                                    TableCell { class: "max-w-[200px] truncate".to_string(), "{scope}" }
                                                    TableCell { "{human_name}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{created}" }
                                                    TableCell { class: "text-right".to_string(),
                                                        Button {
                                                            variant: ButtonVariant::Ghost,
                                                            size: ButtonSize::Sm,
                                                            disabled: row_in_flight,
                                                            onclick: {
                                                                let id = id_for_finish.clone();
                                                                move |_| pending_finish.set(Some(id.clone()))
                                                            },
                                                            {t("coauth.oauth2_sessions.finish")}
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

            DangerousActionDialog {
                open: pending_finish.read().is_some(),
                title: t("coauth.oauth2_sessions.finish"),
                description: "Finish this OAuth2 session now? This can interrupt the third-party client using it.".to_string(),
                confirmation_phrase: confirmation_suffix(pending_finish.read().as_deref().unwrap_or(""), 4),
                confirm_text: t("coauth.oauth2_sessions.finish"),
                cancel_text: t("common.cancel"),
                on_cancel: move |_| pending_finish.set(None),
                on_confirm: move |_| {
                    if let Some(id) = pending_finish.read().clone() {
                        if in_flight.read().is_some() {
                            return;
                        }
                        in_flight.set(Some(id.clone()));
                        spawn(async move {
                            match coauth::finish_oauth2_session(&id).await {
                                Ok(_) => {
                                    show_toast(&t("coauth.oauth2_sessions.toast_finished"), ToastVariant::Success);
                                    data.restart();
                                }
                                Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                            }
                            in_flight.set(None);
                        });
                    }
                    pending_finish.set(None);
                },
            }
        }
    }
}
