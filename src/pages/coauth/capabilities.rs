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
pub fn AuthzCapabilitiesPage() -> Element {
    let mut page = use_signal(|| 1u64);
    let page_val = *page.read();
    let mut data =
        use_resource(
            move || async move { capabilities::list_capabilities(page_val, PAGE_SIZE).await },
        );

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("authz_caps.title"),
                description: t("authz_caps.subtitle"),
            }

            match &*data.read() {
                Some(Ok(resp)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("capabilities.id")} }
                                    TableHead { {t("authz_caps.holder")} }
                                    TableHead { {t("authz_caps.scope")} }
                                    TableHead { {t("authz_caps.status")} }
                                    TableHead { {t("authz_caps.granted_at")} }
                                    TableHead { {t("authz_caps.expires_at")} }
                                }
                            }
                            TableBody {
                                if resp.data.is_empty() {
                                    TableRow {
                                        TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                            {t("authz_caps.empty_title")}
                                        }
                                    }
                                } else {
                                    for cap in resp.data.iter() {
                                        {
                                            let id = cap.id.to_string();
                                            let holder = cap.subject_display();
                                            let scope = cap.resources_display();
                                            let granted = cap.issued_at.to_rfc3339();
                                            let expires = cap.expires_at.map(|at| at.to_rfc3339()).unwrap_or_else(|| "-".to_string());
                                            let revoked = cap.is_revoked();
                                            let status = if revoked { t("capabilities.revoked") } else { t("capabilities.active") };
                                            let variant = if revoked { BadgeVariant::Destructive } else { BadgeVariant::Success };
                                            rsx! {
                                                TableRow {
                                                    key: "{id}",
                                                    TableCell { class: "font-mono text-xs max-w-[220px] truncate".to_string(), "{id}" }
                                                    TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{holder}" }
                                                    TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{scope}" }
                                                    TableCell { Badge { variant, "{status}" } }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{granted}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{expires}" }
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
