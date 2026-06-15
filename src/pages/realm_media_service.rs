//! Realm `media_service` read-only operations view.
//!
//! Renders the effective `ck.component.realm.media_service.v1` cell for a
//! single Realm: the media service DID plus each focus in `foci[]`
//! (`focus_id`, `backend`, `connect_url`, `issuer_kid`, `audience`,
//! `regions`).
//!
//! This view is **read-only** in sodmin: mutating `media_service` is a
//! general-management action that strands through events / yougen, not
//! the operations console. The page therefore only pulls the effective
//! cell via `GET /_soland/admin/realms/{id}/media-service` and renders it.

use dioxus::prelude::*;

use crate::api::media_service;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::{BreadcrumbItem, Breadcrumbs, PageHeader};
use crate::router::Route;
use crate::types::MediaServiceFocus;
use crate::utils::i18n::t;

/// The focus backends accepted by the cokret-spec v3 media_service
/// binding profile (`ck.profile.media_service_binding.v1`). Used only to
/// flag an unrecognized backend in the read-only view.
pub const FOCUS_BACKENDS: &[&str] = &[
    "livekit",
    "mediasoup",
    "janus",
    "cokret-native",
    "moq-relay",
];

#[component]
pub fn RealmMediaService(realm_id: String) -> Element {
    if is_placeholder_resource_id(&realm_id) {
        return selection_required_state("Realm");
    }

    let realm_id_label = realm_id.clone();
    let id_for_fetch = realm_id.clone();
    let mut media_data = use_resource(move || {
        let id = id_for_fetch.clone();
        async move { media_service::get_realm_media_service(&id).await }
    });

    rsx! {
        div { class: "space-y-6",
            Breadcrumbs {
                items: vec![
                    BreadcrumbItem { label: t("delivery_binding.realm"), route: Some(Route::RealmDeliveryBinding { realm_id: realm_id.clone() }) },
                    BreadcrumbItem { label: t("media_service.title"), route: None },
                ],
            }

            PageHeader {
                title: t("media_service.title"),
                description: format!("{}: {}", t("delivery_binding.realm"), realm_id_label),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| media_data.restart(),
                    {t("common.refresh")}
                }
            }

            // Read-only operations view. Realm media_service writes strand
            // through events / yougen, not sodmin.
            div {
                class: "rounded-md border bg-muted/40 px-3 py-2 text-sm text-muted-foreground",
                role: "note",
                {t("media_service.readonly_note")}
            }

            match &*media_data.read() {
                Some(Ok(cell)) => {
                    let service_id = cell.service_id.clone().unwrap_or_else(|| "-".to_string());
                    let allowed = cell.e2ee_key_sources_allowed.clone();
                    let foci = cell.foci.clone();
                    rsx! {
                        Card {
                            CardHeader {
                                CardTitle { {t("media_service.cell_title")} }
                                CardDescription { {t("media_service.subtitle")} }
                            }
                            CardContent {
                                div { class: "space-y-4",
                                    div { class: "space-y-1",
                                        p { class: "text-xs text-muted-foreground", {t("media_service.service_id")} }
                                        p { class: "text-sm font-mono break-all", "{service_id}" }
                                    }
                                    div { class: "space-y-2",
                                        p { class: "text-xs text-muted-foreground", {t("media_service.e2ee_sources")} }
                                        if allowed.is_empty() {
                                            p { class: "text-sm text-muted-foreground", {t("media_service.e2ee_sources_empty")} }
                                        } else {
                                            div { class: "flex flex-wrap gap-1",
                                                for src in allowed.iter() {
                                                    Badge {
                                                        variant: BadgeVariant::Secondary,
                                                        class: "font-mono text-xs".to_string(),
                                                        "{src}"
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        Card {
                            CardHeader {
                                CardTitle { {t("media_service.foci_label")} }
                            }
                            CardContent {
                                div { class: "space-y-3",
                                    if foci.is_empty() {
                                        p { class: "text-sm text-muted-foreground", {t("media_service.empty")} }
                                    } else {
                                        for (idx, focus) in foci.iter().enumerate() {
                                            {render_focus_row(idx, focus)}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! {
                    ErrorBanner { message: e.message.clone(), on_retry: move |_| media_data.restart() }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}

fn render_focus_row(idx: usize, focus: &MediaServiceFocus) -> Element {
    let focus_id = focus.focus_id.clone().unwrap_or_else(|| "-".to_string());
    let backend = focus.backend.clone().unwrap_or_else(|| "-".to_string());
    let connect_url = focus.connect_url.clone().unwrap_or_else(|| "-".to_string());
    let issuer_kid = focus.issuer_kid.clone().unwrap_or_else(|| "-".to_string());
    let audience = focus.audience.clone().unwrap_or_else(|| "-".to_string());
    let regions = focus.regions.clone();
    let known_backend = focus
        .backend
        .as_deref()
        .map(|b| FOCUS_BACKENDS.contains(&b))
        .unwrap_or(true);

    rsx! {
        div {
            class: "rounded-md border p-3 space-y-2",
            div { class: "flex items-center justify-between",
                p { class: "text-xs text-muted-foreground", "focus #{idx + 1}" }
                if !known_backend {
                    Badge { variant: BadgeVariant::Destructive, {t("media_service.unknown_backend")} }
                }
            }
            div { class: "grid gap-2 md:grid-cols-2",
                div { class: "space-y-1",
                    p { class: "text-xs text-muted-foreground", {t("media_service.focus_id")} }
                    p { class: "text-sm font-mono break-all", "{focus_id}" }
                }
                div { class: "space-y-1",
                    p { class: "text-xs text-muted-foreground", {t("media_service.focus_backend")} }
                    p { class: "text-sm font-mono", "{backend}" }
                }
                div { class: "space-y-1 md:col-span-2",
                    p { class: "text-xs text-muted-foreground", {t("media_service.focus_connect_url")} }
                    p { class: "text-sm font-mono break-all", "{connect_url}" }
                }
                div { class: "space-y-1",
                    p { class: "text-xs text-muted-foreground", {t("media_service.focus_issuer_kid")} }
                    p { class: "text-sm font-mono break-all", "{issuer_kid}" }
                }
                div { class: "space-y-1",
                    p { class: "text-xs text-muted-foreground", {t("media_service.focus_audience")} }
                    p { class: "text-sm font-mono break-all", "{audience}" }
                }
                div { class: "space-y-1 md:col-span-2",
                    p { class: "text-xs text-muted-foreground", {t("media_service.focus_regions")} }
                    if regions.is_empty() {
                        p { class: "text-sm text-muted-foreground", "-" }
                    } else {
                        div { class: "flex flex-wrap gap-1",
                            for region in regions.iter() {
                                Badge {
                                    variant: BadgeVariant::Secondary,
                                    class: "font-mono text-xs".to_string(),
                                    "{region}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
