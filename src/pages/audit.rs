use dioxus::prelude::*;

use crate::api::audit;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn AuditLog() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut expanded = use_signal(|| None::<String>);

    let page_val = *page.read();

    let mut data =
        use_resource(move || async move { audit::list_audit_entries(page_val, PAGE_SIZE).await });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("audit.title"),
                description: t("audit.subtitle"),
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("audit.id")} }
                                    TableHead { {t("audit.action")} }
                                    TableHead { {t("audit.actor_id")} }
                                    TableHead { {t("audit.target_type")} }
                                    TableHead { {t("audit.target_id")} }
                                    TableHead { {t("audit.timestamp")} }
                                    TableHead { {t("audit.source_ip")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("audit.no_entries")}
                                        }
                                    }
                                } else {
                                    for entry in resp.data.iter() {
                                        {
                                            let id = entry.id.clone();
                                            let action = entry.action.clone();
                                            let actor_id = entry.actor_id.clone().unwrap_or_else(|| "-".to_string());
                                            let target_type = entry.target_type.clone().unwrap_or_else(|| "-".to_string());
                                            let target_id = entry.target_id.clone().unwrap_or_else(|| "-".to_string());
                                            let timestamp = entry.timestamp.clone().unwrap_or_else(|| "-".to_string());
                                            let source_ip = entry.source_ip.clone().unwrap_or_else(|| "-".to_string());
                                            let details = entry.details.clone();
                                            let is_expanded = expanded.read().as_ref() == Some(&id);

                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(),
                                                        button {
                                                            class: "text-left w-full cursor-pointer",
                                                            onclick: {
                                                                let id = id.clone();
                                                                move |_| {
                                                                    if expanded.read().as_ref() == Some(&id) {
                                                                        expanded.set(None);
                                                                    } else {
                                                                        expanded.set(Some(id.clone()));
                                                                    }
                                                                }
                                                            },
                                                            "{id}"
                                                        }
                                                    }
                                                    TableCell { "{action}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{actor_id}" }
                                                    TableCell { "{target_type}" }
                                                    TableCell { class: "max-w-[150px] truncate".to_string(), "{target_id}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{timestamp}" }
                                                    TableCell { "{source_ip}" }
                                                }
                                                if is_expanded {
                                                    TableRow {
                                                        TableCell { colspan: 99, class: "p-0".to_string(),
                                                            div { class: "p-4 bg-muted/50",
                                                                p { class: "text-xs font-medium mb-2", "Details" }
                                                                pre { class: "text-xs font-mono bg-muted p-3 rounded overflow-auto max-h-64",
                                                                    {details.map(|d| serde_json::to_string_pretty(&d).unwrap_or_else(|_| "{}".to_string())).unwrap_or_else(|| "No details".to_string())}
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
