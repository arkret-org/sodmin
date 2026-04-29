use dioxus::prelude::*;

use crate::api::reports;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::router::Route;
use crate::types::UpdateReportRequest;
use crate::utils::i18n::t;

#[component]
pub fn ReportShow(report_id: String) -> Element {
    let nav = use_navigator();
    let mut show_delete_dialog = use_signal(|| false);
    let mut status_loading = use_signal(|| false);

    let report_id_breadcrumb = report_id.clone();
    let mut report_data = use_resource(move || {
        let id = report_id.clone();
        async move { reports::get_report(&id).await }
    });

    rsx! {
        div { class: "space-y-6",
            Breadcrumbs {
                items: vec![
                    BreadcrumbItem { label: t("reports.title"), route: Some(Route::ReportList {}) },
                    BreadcrumbItem { label: format!("Report #{report_id_breadcrumb}"), route: None },
                ],
            }

            {
                let report_read = report_data.read();
                match &*report_read {
                Some(Ok(report)) => {
                    let id = report.id.clone();
                    let id_for_resolve = id.clone();
                    let id_for_dismiss = id.clone();
                    let id_for_delete = id.clone();
                    let status = report.status.clone().unwrap_or_else(|| "-".to_string());
                    let space_id = report.space_id.clone().unwrap_or_else(|| "-".to_string());
                    let reporter_id = report.reporter_id.clone().unwrap_or_else(|| "-".to_string());
                    let reason = report.reason.clone().unwrap_or_else(|| "-".to_string());
                    let created_at = report.created_at.clone().unwrap_or_else(|| "-".to_string());
                    let is_status_loading = *status_loading.read();

                    rsx! {
                        PageHeader {
                            title: format!("Report #{id}"),
                            description: format!("Status: {status}"),
                            div { class: "flex gap-2",
                                Button {
                                    variant: ButtonVariant::Outline,
                                    disabled: is_status_loading || status == "resolved",
                                    onclick: move |_| {
                                        let id = id_for_resolve.clone();
                                        status_loading.set(true);
                                        spawn(async move {
                                            let req = UpdateReportRequest { status: Some("resolved".to_string()) };
                                            match reports::update_report(&id, &req).await {
                                                Ok(_) => {
                                                    show_toast("Report resolved", ToastVariant::Success);
                                                    report_data.restart();
                                                }
                                                Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                            }
                                            status_loading.set(false);
                                        });
                                    },
                                    {t("reports.resolve")}
                                }
                                Button {
                                    variant: ButtonVariant::Outline,
                                    disabled: is_status_loading || status == "dismissed",
                                    onclick: move |_| {
                                        let id = id_for_dismiss.clone();
                                        status_loading.set(true);
                                        spawn(async move {
                                            let req = UpdateReportRequest { status: Some("dismissed".to_string()) };
                                            match reports::update_report(&id, &req).await {
                                                Ok(_) => {
                                                    show_toast("Report dismissed", ToastVariant::Success);
                                                    report_data.restart();
                                                }
                                                Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                            }
                                            status_loading.set(false);
                                        });
                                    },
                                    {t("reports.dismiss")}
                                }
                                Button {
                                    variant: ButtonVariant::Destructive,
                                    onclick: move |_| show_delete_dialog.set(true),
                                    {t("common.delete")}
                                }
                            }
                        }

                        Card {
                            CardHeader { CardTitle { {t("reports.report_details")} } }
                            CardContent {
                                div { class: "space-y-4",
                                    InfoRow { label: t("reports.id"), value: id }
                                    InfoRow { label: t("reports.status"), value: status }
                                    InfoRow { label: t("reports.space_id"), value: space_id }
                                    InfoRow { label: t("reports.reporter_id"), value: reporter_id }
                                    InfoRow { label: t("reports.reason"), value: reason }
                                    InfoRow { label: t("reports.created_at"), value: created_at }
                                }
                            }
                        }

                        ConfirmDialog {
                            open: *show_delete_dialog.read(),
                            title: t("reports.delete_report"),
                            description: "Are you sure? This action cannot be undone.".to_string(),
                            confirm_text: t("common.delete"),
                            destructive: true,
                            on_confirm: move |_| {
                                show_delete_dialog.set(false);
                                let id = id_for_delete.clone();
                                spawn(async move {
                                    match reports::delete_report(&id).await {
                                        Ok(_) => {
                                            show_toast("Report deleted", ToastVariant::Success);
                                            nav.push(Route::ReportList {});
                                        }
                                        Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                                    }
                                });
                            },
                            on_cancel: move |_| show_delete_dialog.set(false),
                        }
                    }
                },
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        on_retry: move |_| report_data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
            }
        }
    }
}

#[component]
fn InfoRow(label: String, value: String) -> Element {
    rsx! {
        div { class: "flex items-center justify-between py-2",
            span { class: "text-sm font-medium text-muted-foreground", "{label}" }
            span { class: "text-sm max-w-[60%] text-right break-all", "{value}" }
        }
    }
}
