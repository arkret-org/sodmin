// The publish endpoint requires a populated request body (template_key
// + channel + body_template at minimum); a dedicated editor lands in
// a follow-up. The wire surface is reachable from
// `coauth::publish_notification_template` /
// `coauth::CoauthPublishTemplateRequest` once that editor exists.
use dioxus::prelude::*;

use crate::api::coauth;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::utils::i18n::t;

#[component]
pub fn NotificationTemplatesPage() -> Element {
    let mut data = use_resource(|| async { coauth::list_notification_templates().await });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("coauth.notification_templates.title"),
                description: t("coauth.notification_templates.subtitle"),
            }

            match &*data.read() {
                Some(Ok(templates)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("coauth.notification_templates.key")} }
                                    TableHead { {t("coauth.notification_templates.description")} }
                                }
                            }
                            TableBody {
                                if templates.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("coauth.notification_templates.no_templates")}
                                        }
                                    }
                                } else {
                                    for tmpl in templates.iter() {
                                        {
                                            let key = tmpl.key.clone();
                                            let description = tmpl.description.clone();

                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(), "{key}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{description}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
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
