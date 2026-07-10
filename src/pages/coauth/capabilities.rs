use dioxus::prelude::*;

use crate::api::capabilities;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::CursorPagination;
use crate::components::ui::table::*;
use crate::types::CapabilitySummaryExt;
use crate::utils::i18n::t;

const PAGE_SIZE: u64 = 25;

#[component]
pub fn AuthzCapabilitiesPage() -> Element {
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);
    let mut data = use_resource(move || {
        let cursor = cursor_snapshot.clone();
        async move { capabilities::list_capabilities(cursor.as_deref(), PAGE_SIZE).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("authz_caps.title"),
                description: t("authz_caps.subtitle"),
            }

            match &*data.read() {
                Some(Ok(resp)) => {
                    let next_cursor = resp.next_cursor.clone();
                    let stack_depth = cursor_stack.read().len();
                    rsx! {
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
                                    if resp.capabilities.is_empty() {
                                        TableRow {
                                            TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                                {t("authz_caps.empty_title")}
                                            }
                                        }
                                    } else {
                                        for cap in resp.capabilities.iter() {
                                            {
                                                let id = cap.grant_id.clone();
                                                let holder = cap.subject.clone();
                                                let scope = cap.resource_display();
                                                let granted = cap.created_at.map(|at| at.to_rfc3339()).unwrap_or_else(|| "-".to_string());
                                                let expires = cap.expires_at.map(|at| at.to_rfc3339()).unwrap_or_else(|| "-".to_string());
                                                let revoked = cap.revoked;
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

                        CursorPagination {
                            depth: stack_depth,
                            has_next: next_cursor.is_some(),
                            on_prev: move |_| {
                                let mut new_stack = cursor_stack.read().clone();
                                if new_stack.len() > 1 {
                                    new_stack.pop();
                                    cursor_stack.set(new_stack);
                                }
                            },
                            on_next: move |_| {
                                if let Some(c) = next_cursor.clone() {
                                    let mut new_stack = cursor_stack.read().clone();
                                    new_stack.push(Some(c));
                                    cursor_stack.set(new_stack);
                                }
                            },
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
