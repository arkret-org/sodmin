use dioxus::prelude::*;

use crate::api::capabilities;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::*;
use crate::types::CapabilityGrantExt;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn CapabilityList() -> Element {
    let mut page = use_signal(|| 1u64);
    let page_val = *page.read();

    let mut data =
        use_resource(
            move || async move { capabilities::list_capabilities(page_val, PAGE_SIZE).await },
        );

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("capabilities.title"),
                description: t("capabilities.subtitle"),
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("capabilities.id")} }
                                    TableHead { {t("capabilities.grantor_id")} }
                                    TableHead { {t("capabilities.grantee_id")} }
                                    TableHead { {t("capabilities.capability")} }
                                    TableHead { {t("capabilities.scope")} }
                                    TableHead { {t("capabilities.granted_at")} }
                                    TableHead { {t("capabilities.expires_at")} }
                                    TableHead { {t("capabilities.revoked")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("capabilities.no_capabilities")}
                                        }
                                    }
                                } else {
                                    for cap in resp.data.iter() {
                                        {
                                            let id = cap.id.to_string();
                                            let issuer = cap.issuer.to_string();
                                            let subject = cap.subject_display();
                                            let actions = cap.actions_display();
                                            let resources = cap.resources_display();
                                            let issued_at = cap.issued_at.to_rfc3339();
                                            let expires = cap.expires_at.map(|at| at.to_rfc3339()).unwrap_or_else(|| "-".to_string());
                                            let revoked = cap.is_revoked();
                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-medium".to_string(), "{id}" }
                                                    TableCell { class: "max-w-[180px] truncate".to_string(), "{issuer}" }
                                                    TableCell { class: "max-w-[180px] truncate".to_string(), "{subject}" }
                                                    TableCell { "{actions}" }
                                                    TableCell { class: "max-w-[260px] truncate".to_string(), "{resources}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{issued_at}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{expires}" }
                                                    TableCell {
                                                        if revoked {
                                                            Badge { variant: BadgeVariant::Destructive, {t("capabilities.revoked")} }
                                                        } else {
                                                            Badge { variant: BadgeVariant::Success, {t("capabilities.active")} }
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
                        total: resp.total_or_len(),
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
