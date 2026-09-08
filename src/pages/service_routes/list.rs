//! Paginated local service-route inventory.

use dioxus::prelude::*;

use crate::api::service_routes;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::pagination::CursorPagination;
use crate::components::ui::table::*;
use crate::router::Route;
use crate::utils::i18n::t;

const PAGE_SIZE: usize = 25;

#[component]
pub fn ServiceRouteList() -> Element {
    let mut cursors = use_signal(|| vec![None::<String>]);
    let cursor = cursors.read().last().cloned().unwrap_or(None);
    let mut routes = use_resource(move || {
        let cursor = cursor.clone();
        async move { service_routes::list_service_routes(cursor.as_deref(), PAGE_SIZE).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("service_routes.title"),
                description: t("service_routes.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| routes.restart(),
                    {t("common.refresh")}
                }
            }

            match &*routes.read() {
                Some(Ok(data)) => {
                    let next_cursor = data.next_cursor.clone();
                    let depth = cursors.read().len();
                    rsx! {
                        div { class: "rounded-md border",
                            Table {
                                TableHeader {
                                    TableRow {
                                        TableHead { {t("service_routes.service_id")} }
                                        TableHead { {t("service_routes.kind")} }
                                        TableHead { {t("service_routes.version")} }
                                        TableHead { {t("service_routes.cache_expiry")} }
                                        TableHead { {t("service_routes.status")} }
                                    }
                                }
                                TableBody {
                                    if data.routes.is_empty() {
                                        TableRow {
                                            TableCell {
                                                colspan: 99,
                                                class: "text-center text-muted-foreground py-8".to_string(),
                                                {t("service_routes.empty")}
                                            }
                                        }
                                    } else {
                                        for route in data.routes.iter() {
                                            {
                                                let service_id = route.service_id.to_string();
                                                let service_kind = route.service_kind.clone();
                                                let method_version = route.version_id.clone()
                                                    .unwrap_or_else(|| t("service_routes.unknown"));
                                                let cache_expiry = route.cache_expires_at
                                                    .map(|value| value.to_rfc3339())
                                                    .unwrap_or_else(|| t("service_routes.unknown"));
                                                rsx! {
                                                    TableRow { key: "{service_id}:{service_kind}",
                                                        TableCell {
                                                            Link {
                                                                to: Route::ServiceRouteShow {
                                                                    service_id: urlencoding::encode(&service_id).to_string(),
                                                                    service_kind: urlencoding::encode(&service_kind).to_string(),
                                                                },
                                                                class: "font-mono text-xs text-primary hover:underline",
                                                                "{service_id}"
                                                            }
                                                        }
                                                        TableCell { class: "font-mono text-xs".to_string(), "{service_kind}" }
                                                        TableCell { class: "font-mono text-xs".to_string(), "{method_version}" }
                                                        TableCell { class: "font-mono text-xs".to_string(), "{cache_expiry}" }
                                                        TableCell {
                                                            if route.quarantined {
                                                                Badge { variant: BadgeVariant::Destructive, {t("service_routes.quarantined")} }
                                                            } else if route.known {
                                                                Badge { variant: BadgeVariant::Success, {t("service_routes.known")} }
                                                            } else {
                                                                Badge { variant: BadgeVariant::Secondary, {t("service_routes.unknown")} }
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
                            depth,
                            has_next: next_cursor.is_some(),
                            on_prev: move |_| {
                                let mut stack = cursors.read().clone();
                                if stack.len() > 1 {
                                    stack.pop();
                                    cursors.set(stack);
                                }
                            },
                            on_next: move |_| {
                                if let Some(cursor) = next_cursor.clone() {
                                    let mut stack = cursors.read().clone();
                                    stack.push(Some(cursor));
                                    cursors.set(stack);
                                }
                            },
                        }
                    }
                },
                Some(Err(error)) => rsx! {
                    ErrorBanner { message: error.message.clone(), on_retry: move |_| routes.restart() }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}
