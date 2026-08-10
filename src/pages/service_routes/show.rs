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
use crate::components::ui::table::*;
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
                                        if let Some(current) = detail.current_record.as_ref() {
                                            InfoRow { label: t("service_routes.full_id"), value: current.full_id.clone() }
                                            InfoRow { label: t("service_routes.history_head"), value: current.method_history_head.clone() }
                                            InfoRow { label: t("service_routes.version"), value: current.version_id.clone() }
                                            InfoRow { label: t("service_routes.base_url"), value: current.base_url.clone() }
                                            InfoRow { label: t("service_routes.refresh_after"), value: timestamp(current.refresh_after) }
                                            InfoRow { label: t("service_routes.signed_expiry"), value: timestamp(current.signed_expires_at) }
                                        } else {
                                            InfoRow { label: t("service_routes.current_title"), value: t("service_routes.unknown") }
                                        }
                                    }
                                }
                            }

                            div { class: "grid gap-6 lg:grid-cols-2",
                                Card {
                                    CardHeader { CardTitle { {t("service_routes.floor_title")} } }
                                    CardContent {
                                        if let Some(floor) = detail.floor.as_ref() {
                                            div { class: "space-y-4",
                                                InfoRow { label: t("service_routes.sequence"), value: floor.record_sequence.to_string() }
                                                InfoRow { label: t("service_routes.digest"), value: floor.record_digest.clone() }
                                                InfoRow { label: t("service_routes.verified_at"), value: timestamp(floor.verified_at) }
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
                                                InfoRow { label: t("service_routes.cached_at"), value: timestamp(cache.cached_at) }
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
                                    CardTitle { {t("service_routes.notices_title")} }
                                    CardDescription { {t("service_routes.notices_subtitle")} }
                                }
                                CardContent {
                                    if detail.notices.is_empty() {
                                        p { class: "text-sm text-muted-foreground", {t("service_routes.none")} }
                                    } else {
                                        Table {
                                            TableHeader { TableRow {
                                                TableHead { {t("service_routes.handover_id")} }
                                                TableHead { {t("service_routes.revision")} }
                                                TableHead { {t("service_routes.status")} }
                                                TableHead { {t("service_routes.expiry")} }
                                            } }
                                            TableBody {
                                                for notice in detail.notices.iter() {
                                                    TableRow { key: "{notice.handover_id}:{notice.notice_revision}",
                                                        TableCell { class: "font-mono text-xs".to_string(), "{notice.handover_id}" }
                                                        TableCell { "{notice.notice_revision}" }
                                                        TableCell { Badge { variant: BadgeVariant::Secondary, "{notice.state}" } }
                                                        TableCell { class: "font-mono text-xs".to_string(), {timestamp(notice.expires_at)} }
                                                    }
                                                }
                                            }
                                        }
                                        if detail.notices_truncated {
                                            p { class: "mt-3 text-xs text-muted-foreground", {t("service_routes.truncated")} }
                                        }
                                    }
                                }
                            }

                            Card {
                                CardHeader { CardTitle { {t("service_routes.acks_title")} } }
                                CardContent {
                                    if detail.acks.is_empty() {
                                        p { class: "text-sm text-muted-foreground", {t("service_routes.none")} }
                                    } else {
                                        Table {
                                            TableHeader { TableRow {
                                                TableHead { {t("service_routes.request_id")} }
                                                TableHead { {t("service_routes.realm_id")} }
                                                TableHead { {t("service_routes.receiver")} }
                                                TableHead { {t("service_routes.accepted_at")} }
                                            } }
                                            TableBody {
                                                for ack in detail.acks.iter() {
                                                    TableRow { key: "{ack.request_id}",
                                                        TableCell { class: "font-mono text-xs".to_string(), "{ack.request_id}" }
                                                        TableCell { class: "font-mono text-xs".to_string(), "{ack.realm_id}" }
                                                        TableCell { class: "font-mono text-xs".to_string(), "{ack.receiver_service_id}" }
                                                        TableCell { class: "font-mono text-xs".to_string(), {timestamp(ack.accepted_at)} }
                                                    }
                                                }
                                            }
                                        }
                                        if detail.acks_truncated {
                                            p { class: "mt-3 text-xs text-muted-foreground", {t("service_routes.truncated")} }
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
