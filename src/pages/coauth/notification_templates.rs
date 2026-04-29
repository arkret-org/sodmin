use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn NotificationTemplatesPage() -> Element {
    let mut page = use_signal(|| 1u64);
    let mut publish_loading = use_signal(|| false);

    let page_val = *page.read();

    let mut data = use_resource(move || async move {
        coauth::list_notification_templates(page_val, PAGE_SIZE).await
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("coauth.notification_templates.title"),
                description: t("coauth.notification_templates.subtitle"),
                Button {
                    variant: ButtonVariant::Default,
                    disabled: *publish_loading.read(),
                    onclick: move |_| {
                        publish_loading.set(true);
                        spawn(async move {
                            match coauth::publish_notification_templates().await {
                                Ok(_) => show_toast("Templates published", ToastVariant::Success),
                                Err(e) => show_toast(&format!("Failed: {}", e.message), ToastVariant::Error),
                            }
                            publish_loading.set(false);
                        });
                    },
                    {t("coauth.notification_templates.publish")}
                }
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("coauth.notification_templates.id")} }
                                    TableHead { {t("coauth.notification_templates.name")} }
                                    TableHead { {t("coauth.notification_templates.channel_type")} }
                                    TableHead { {t("coauth.notification_templates.locale")} }
                                    TableHead { {t("coauth.notification_templates.updated_at")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("coauth.notification_templates.no_templates")}
                                        }
                                    }
                                } else {
                                    for tmpl in resp.data.iter() {
                                        {
                                            let id = tmpl.id.clone();
                                            let name = tmpl.name.clone().unwrap_or_else(|| "-".to_string());
                                            let channel_type = tmpl.channel_type.clone().unwrap_or_else(|| "-".to_string());
                                            let locale = tmpl.locale.clone().unwrap_or_else(|| "-".to_string());
                                            let updated = tmpl.updated_at.clone().unwrap_or_else(|| "-".to_string());

                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell { "{name}" }
                                                    TableCell { "{channel_type}" }
                                                    TableCell { "{locale}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{updated}" }
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
