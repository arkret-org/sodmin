use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn AuditLogPage() -> Element {
    let mut page = use_signal(|| 1u64);

    let page_val = *page.read();

    let mut data = use_resource(move || async move {
        coauth::list_audit_feed(page_val, PAGE_SIZE).await
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("coauth.audit_log.title"),
                description: t("coauth.audit_log.subtitle"),
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("coauth.audit_log.id")} }
                                    TableHead { {t("coauth.audit_log.operation")} }
                                    TableHead { {t("coauth.audit_log.actor")} }
                                    TableHead { {t("coauth.audit_log.target_type")} }
                                    TableHead { {t("coauth.audit_log.target_id")} }
                                    TableHead { {t("coauth.audit_log.timestamp")} }
                                    TableHead { {t("coauth.audit_log.source_ip")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("coauth.audit_log.no_entries")}
                                        }
                                    }
                                } else {
                                    for entry in resp.data.iter() {
                                        {
                                            let id = entry.id.clone();
                                            let op = entry.operation.clone();
                                            let actor = entry.actor_user_id.clone().unwrap_or_else(|| "-".to_string());
                                            let target_type = entry.target_type.clone().unwrap_or_else(|| "-".to_string());
                                            let target_id = entry.target_id.clone().unwrap_or_else(|| "-".to_string());
                                            let ts = entry.timestamp.clone().unwrap_or_else(|| "-".to_string());
                                            let ip = entry.source_ip.clone().unwrap_or_else(|| "-".to_string());

                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell { "{op}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{actor}" }
                                                    TableCell { "{target_type}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{target_id}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{ts}" }
                                                    TableCell { "{ip}" }
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
    }
}
