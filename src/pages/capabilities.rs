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
pub fn CapabilityList() -> Element {
    let mut cursor_stack = use_signal(|| vec![None::<String>]);
    let cursor_snapshot = cursor_stack.read().last().cloned().unwrap_or(None);

    let mut data = use_resource(move || {
        let cursor = cursor_snapshot.clone();
        async move { capabilities::list_capabilities(cursor.as_deref(), PAGE_SIZE).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("capabilities.title"),
                description: t("capabilities.subtitle"),
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
                                    if resp.capabilities.is_empty() {
                                        TableRow {
                                            TableCell { class: "text-center text-muted-foreground py-8".to_string(), colspan: 99,
                                                {t("capabilities.no_capabilities")}
                                            }
                                        }
                                    } else {
                                        for cap in resp.capabilities.iter() {
                                            {
                                                let id = cap.grant_id.clone();
                                                let issuer = cap.issuer_id.clone();
                                                let subject_id = cap.subject_id.clone();
                                                let actions = cap.actions_display();
                                                let resource = cap.resource_display();
                                                let issued_at = cap.created_at.map(|at| at.to_rfc3339()).unwrap_or_else(|| "-".to_string());
                                                let expires = cap.expires_at.map(|at| at.to_rfc3339()).unwrap_or_else(|| "-".to_string());
                                                let revoked = cap.revoked;
                                                rsx! {
                                                    TableRow {
                                                        key: "{id}",
                                                        TableCell { class: "font-medium".to_string(), "{id}" }
                                                        TableCell { class: "max-w-[180px] truncate".to_string(), "{issuer}" }
                                                        TableCell { class: "max-w-[180px] truncate".to_string(), "{subject_id}" }
                                                        TableCell { "{actions}" }
                                                        TableCell { class: "max-w-[260px] truncate".to_string(), "{resource}" }
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
