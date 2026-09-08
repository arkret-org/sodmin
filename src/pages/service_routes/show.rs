//! Detail view of one locally verified service-route projection.

use dioxus::prelude::*;

use crate::api::service_routes;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::info_row::InfoRow;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::router::Route;
use crate::utils::i18n::t;

fn timestamp(value: chrono::DateTime<chrono::Utc>) -> String {
    value.to_rfc3339()
}

#[component]
pub fn ServiceRouteShow(service_id: String, service_kind: String) -> Element {
    let decoded_service_id = urlencoding::decode(&service_id)
        .map(|value| value.into_owned())
        .unwrap_or(service_id);
    let decoded_service_kind = urlencoding::decode(&service_kind)
        .map(|value| value.into_owned())
        .unwrap_or(service_kind);
    let id_for_resource = decoded_service_id.clone();
    let kind_for_resource = decoded_service_kind.clone();
    let mut route = use_resource(move || {
        let service_id = id_for_resource.clone();
        let service_kind = kind_for_resource.clone();
        async move { service_routes::get_service_route(&service_id, &service_kind).await }
    });

    rsx! {
        div { class: "space-y-6",
            Breadcrumbs {
                items: vec![
                    BreadcrumbItem { label: t("service_routes.title"), route: Some(Route::ServiceRouteList {}) },
                    BreadcrumbItem { label: decoded_service_id.clone(), route: None },
                ],
            }

            match &*route.read() {
                Some(Ok(detail)) => {
                    let observed_at = timestamp(detail.observed_at);
                    rsx! {
                        PageHeader {
                            title: detail.service_id.to_string(),
                            description: format!("{} · {}", detail.service_kind, t("service_routes.local_only")),
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| route.restart(),
                                {t("common.refresh")}
                            }
                        }

                        if !detail.known {
                            Card {
                                CardContent {
                                    div { class: "py-8 text-center space-y-2",
                                        Badge { variant: BadgeVariant::Secondary, {t("service_routes.unknown")} }
                                        p { class: "text-sm text-muted-foreground", {t("service_routes.unknown_detail")} }
                                    }
                                }
                            }
                        } else {
                            Card {
                                CardHeader { CardTitle { {t("service_routes.current_title")} } }
                                CardContent {
                                    div { class: "space-y-4",
                                        InfoRow { label: t("service_routes.service_id"), value: detail.service_id.to_string() }
                                        InfoRow { label: t("service_routes.kind"), value: detail.service_kind.clone() }
                                        InfoRow { label: t("service_routes.authority"), value: detail.authority.clone() }
                                        InfoRow { label: t("service_routes.observed_at"), value: observed_at }
                                        if let Some(current) = detail.current_route.as_ref() {
                                            InfoRow { label: t("service_routes.did"), value: current.did.clone() }
                                            InfoRow { label: t("service_routes.history_head"), value: current.method_history_head.clone() }
                                            InfoRow { label: t("service_routes.version"), value: current.version_id.clone() }
                                            InfoRow { label: t("service_routes.base_url"), value: current.base_url.clone() }
                                        } else {
                                            InfoRow { label: t("service_routes.current_title"), value: t("service_routes.unknown") }
                                        }
                                    }
                                }
                            }

                            div { class: "grid gap-6 lg:grid-cols-2",
                                Card {
                                    CardHeader { CardTitle { {t("service_routes.method_state_title")} } }
                                    CardContent {
                                        if let Some(method_state) = detail.method_state.as_ref() {
                                            div { class: "space-y-4",
                                                InfoRow { label: t("service_routes.version"), value: method_state.version_id.clone() }
                                                InfoRow { label: t("service_routes.history_head"), value: method_state.method_history_head.clone() }
                                                InfoRow { label: t("service_routes.verified_at"), value: timestamp(method_state.verified_at) }
                                            }
                                        } else {
                                            p { class: "text-sm text-muted-foreground", {t("service_routes.unknown")} }
                                        }
                                    }
                                }
                                Card {
                                    CardHeader { CardTitle { {t("service_routes.cache_title")} } }
                                    CardContent {
                                        if let Some(cache) = detail.cache.as_ref() {
                                            div { class: "space-y-4",
                                                InfoRow { label: t("service_routes.verified_at"), value: timestamp(cache.verified_at) }
                                                InfoRow { label: t("service_routes.cache_expiry"), value: timestamp(cache.cache_expires_at) }
                                                InfoRow {
                                                    label: t("service_routes.routable"),
                                                    value: if cache.routable_at_observed_at { t("common.yes") } else { t("common.no") },
                                                }
                                            }
                                        } else {
                                            p { class: "text-sm text-muted-foreground", {t("service_routes.unknown")} }
                                        }
                                    }
                                }
                            }

                            Card {
                                CardHeader {
                                    CardTitle { {t("service_routes.quarantine_title")} }
                                    CardDescription { {t("service_routes.private_diagnostics")} }
                                }
                                CardContent {
                                    if detail.quarantine.is_empty() {
                                        p { class: "text-sm text-muted-foreground", {t("service_routes.none")} }
                                    } else {
                                        div { class: "space-y-4",
                                            for item in detail.quarantine.iter() {
                                                div { class: "rounded-md border border-destructive/40 p-4 space-y-2",
                                                    Badge { variant: BadgeVariant::Destructive, {t("service_routes.quarantined")} }
                                                    p { class: "font-mono text-xs break-all", "{item.artifact_family} / {item.artifact_key}" }
                                                    p { class: "font-mono text-xs break-all text-muted-foreground", "{item.accepted_digest} → {item.conflicting_digest}" }
                                                    pre { class: "overflow-auto rounded bg-muted p-3 text-xs", {serde_json::to_string_pretty(&item.diagnostic).unwrap_or_else(|_| "{}".to_string())} }
                                                }
                                            }
                                        }
                                        if detail.quarantine_truncated {
                                            p { class: "mt-3 text-xs text-muted-foreground", {t("service_routes.truncated")} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                Some(Err(error)) => rsx! {
                    ErrorBanner { message: error.message.clone(), on_retry: move |_| route.restart() }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}
