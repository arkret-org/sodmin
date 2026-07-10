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
pub fn UpstreamLinksPage() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut show_delete = use_signal(|| None::<String>);

    let mut data = use_resource(move || async move {
        let page_val = *page.read();
        coauth::list_upstream_links(page_val, PAGE_SIZE).await
    });
    let page_val = *page.read();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("coauth.upstream_links.title"),
                description: t("coauth.upstream_links.subtitle"),
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("coauth.upstream_links.id")} }
                                    TableHead { {t("coauth.upstream_links.user_id")} }
                                    TableHead { {t("coauth.upstream_links.provider_id")} }
                                    TableHead { {t("coauth.upstream_links.subject")} }
                                    TableHead { {t("coauth.upstream_links.created_at")} }
                                    TableHead { class: "text-right".to_string(), {t("common.actions")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("coauth.upstream_links.no_links")}
                                        }
                                    }
                                } else {
                                    for link in resp.data.iter() {
                                        {
                                            let id = link.id.clone();
                                            let user_id = link.user_id.clone().unwrap_or_else(|| "-".to_string());
                                            let provider_id = link.provider_id.clone().unwrap_or_else(|| "-".to_string());
                                            let subject = link.subject.clone().unwrap_or_else(|| "-".to_string());
                                            let created = link.created_at.clone().unwrap_or_else(|| "-".to_string());

                                            let id_for_delete = id.clone();

                                            rsx! {
                                                TableRow {
                                                    key: "{id}",
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{user_id}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{provider_id}" }
                                                    TableCell { "{subject}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{created}" }
                                                    TableCell { class: "text-right".to_string(),
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

        DangerousActionDialog {
            open: show_delete.read().is_some(),
            title: t("common.delete"),
            description: t("coauth.upstream_links.delete_description"),
            confirmation_phrase: confirmation_suffix(show_delete.read().as_deref().unwrap_or(""), 4),
            confirm_text: t("common.delete"),
            cancel_text: t("common.cancel"),
            on_confirm: move |_| {
                if let Some(id) = show_delete.read().clone() {
                    let id = id.clone();
                    spawn(async move {
                        match coauth::delete_upstream_link(&id).await {
                            Ok(_) => {
                                show_toast(&t("coauth.upstream_links.toast_deleted"), ToastVariant::Success);
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
