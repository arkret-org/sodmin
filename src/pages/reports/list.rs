use dioxus::prelude::*;

use crate::api::reports;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::router::Route;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn ReportList() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut status_filter = use_signal(|| "all".to_string());

    let page_val = *page.read();
    let status_val = status_filter.read().clone();

    let mut reports_data = use_resource(move || {
        let status = status_val.clone();
        async move {
            reports::list_reports(page_val, PAGE_SIZE, &status).await
        }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("reports.title"),
                description: t("reports.subtitle"),
            }

            div { class: "flex gap-2",
                Button {
                    variant: if *status_filter.read() == "all" { ButtonVariant::Default } else { ButtonVariant::Outline },
                    onclick: move |_| { status_filter.set("all".to_string()); page.set(1); },
                    {t("reports.all")}
                }
                Button {
                    variant: if *status_filter.read() == "open" { ButtonVariant::Default } else { ButtonVariant::Outline },
                    onclick: move |_| { status_filter.set("open".to_string()); page.set(1); },
                    {t("reports.open")}
                }
                Button {
                    variant: if *status_filter.read() == "resolved" { ButtonVariant::Default } else { ButtonVariant::Outline },
                    onclick: move |_| { status_filter.set("resolved".to_string()); page.set(1); },
                    {t("reports.resolved")}
                }
            }

            match &*reports_data.read() {
                Some(Ok(data)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("reports.id")} }
                                    TableHead { {t("reports.status")} }
                                    TableHead { {t("reports.space_id")} }
                                    TableHead { {t("reports.reporter_id")} }
                                    TableHead { {t("reports.reason")} }
                                    TableHead { {t("reports.created_at")} }
                                }
                            }
                            TableBody {
                                if data.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("reports.no_reports")}
                                        }
                                    }
                                } else {
                                    for report in data.data.iter() {
                                        {
                                            let id = report.id.clone();
                                            let status = report.status.clone().unwrap_or_else(|| "-".to_string());
                                            let space_id = report.space_id.clone().unwrap_or_else(|| "-".to_string());
                                            let reporter_id = report.reporter_id.clone().unwrap_or_else(|| "-".to_string());
                                            let reason = report.reason.clone().unwrap_or_else(|| "-".to_string());
                                            let created_at = report.created_at.clone().unwrap_or_else(|| "-".to_string());

                                            let variant = match status.as_str() {
                                                "resolved" => BadgeVariant::Success,
                                                "open" => BadgeVariant::Default,
                                                _ => BadgeVariant::Secondary,
                                            };

                                            rsx! {
                                                TableRow {
                                                    TableCell {
                                                        Link {
                                                            to: Route::ReportShow { report_id: id.clone() },
                                                            class: "font-medium text-primary hover:underline",
                                                            "#{id}"
                                                        }
                                                    }
                                                    TableCell {
                                                        Badge { variant: variant, "{status}" }
                                                    }
                                                    TableCell { class: "max-w-[200px] truncate".to_string(), "{space_id}" }
                                                    TableCell { class: "max-w-[200px] truncate".to_string(), "{reporter_id}" }
                                                    TableCell { class: "max-w-[300px] truncate".to_string(), "{reason}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{created_at}" }
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
                        total: data.total,
                        per_page: PAGE_SIZE,
                        on_page_change: move |p| page.set(p),
                    }
                },
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        on_retry: move |_| reports_data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}
