//! Read-only "Starid resolver status" panel (round C35.4).
//!
//! Mirrors the upstream starid resolver's `/_arkret/root/identity/describe`
//! canonical `ServiceDescribe` envelope so the operator can inspect the
//! writer identity, protocol version, supported profiles, and Starid
//! product extensions without maintaining a second protocol DTO.
//!
//! The panel is hidden when the deployment hasn't wired
//! `starid_public_url` (see `utils::net::session::has_starid()`); the page
//! itself stays reachable so a deep link from the docs renders a
//! "Starid not configured" empty state with a config-docs pointer.

use dioxus::prelude::*;

use crate::api::starid::{self, StaridDescribe};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::utils::i18n::t;
use crate::utils::net::error::HttpError;

#[component]
pub fn StaridResolverPage() -> Element {
    let mut data = use_resource(|| async { starid::get_describe().await });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("starid_resolver.title"),
                description: t("starid_resolver.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }

            match &*data.read() {
                None => rsx! { PageSkeleton {} },
                Some(Err(_not_configured)) => not_configured_card(),
                Some(Ok(Err(http_error))) => error_card(http_error.clone(), data),
                Some(Ok(Ok(describe))) => describe_card(describe.clone()),
            }
        }
    }
}

fn not_configured_card() -> Element {
    rsx! {
        Card {
            CardHeader {
                div { class: "flex items-center justify-between gap-2",
                    div { class: "space-y-1",
                        CardTitle { class: "text-lg".to_string(), {t("starid_resolver.title")} }
                        CardDescription { {t("starid_resolver.not_configured_hint")} }
                    }
                    Badge { variant: BadgeVariant::Secondary, {t("starid_resolver.not_configured_badge")} }
                }
            }
            CardContent {
                p { class: "text-sm text-muted-foreground", {t("starid_resolver.not_configured_body")} }
                a {
                    class: "text-sm text-primary hover:underline",
                    href: "/docs/starid-resolver.md",
                    target: "_blank",
                    rel: "noreferrer",
                    {t("starid_resolver.docs_link")}
                }
            }
        }
    }
}

fn error_card(
    error: HttpError,
    mut data: Resource<Result<Result<StaridDescribe, HttpError>, starid::StaridNotConfigured>>,
) -> Element {
    rsx! {
        ErrorBanner {
            message: error.message.clone(),
            on_retry: move |_| data.restart(),
        }
    }
}

fn describe_card(describe: StaridDescribe) -> Element {
    let service_id = describe.service_id.as_str().to_string();
    let service_full_id = describe.service_resolution.full_id.as_str().to_string();
    let registry_mode = describe
        .extra_str(&["x_starid_registry_mode"])
        .unwrap_or_else(|| "-".to_string());
    let protocol = describe.protocol_version.clone();
    let methods = describe.extra_string_list(&["x_starid_supported_methods"]);
    let profiles = describe.supported_profiles.clone();

    rsx! {
        Card {
            CardHeader {
                div { class: "flex items-center justify-between gap-2",
                    div { class: "space-y-1",
                        CardTitle { class: "text-lg".to_string(), {t("starid_resolver.card_title")} }
                        CardDescription { {t("starid_resolver.card_subtitle")} }
                    }
                }
            }
            CardContent {
                div { class: "space-y-4",
                    div { class: "grid gap-3 sm:grid-cols-2 lg:grid-cols-3",
                        {info_cell(t("starid_resolver.service_id"), service_id)}
                        {info_cell(t("starid_resolver.service_full_id"), service_full_id)}
                        {info_cell(t("starid_resolver.registry_mode"), registry_mode)}
                        {info_cell(t("starid_resolver.protocol_version"), protocol)}
                    }

                    div { class: "space-y-1",
                        p {
                            class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                            {t("starid_resolver.methods_label")}
                        }
                        if methods.is_empty() {
                            p { class: "text-xs text-muted-foreground", "-" }
                        } else {
                            div { class: "flex flex-wrap gap-1",
                                for m in methods.iter() {
                                    Badge {
                                        variant: BadgeVariant::Secondary,
                                        class: "font-mono text-xs".to_string(),
                                        "{m}"
                                    }
                                }
                            }
                        }
                    }

                    div { class: "space-y-1",
                        p {
                            class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                            {t("starid_resolver.profiles_label")}
                        }
                        if profiles.is_empty() {
                            p { class: "text-xs text-muted-foreground", "-" }
                        } else {
                            div { class: "flex flex-wrap gap-1",
                                for p in profiles.iter() {
                                    Badge {
                                        variant: BadgeVariant::Secondary,
                                        class: "font-mono text-xs".to_string(),
                                        "{p}"
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

fn info_cell(label: String, value: String) -> Element {
    rsx! {
        div {
            p { class: "text-xs text-muted-foreground", "{label}" }
            p { class: "text-sm font-semibold break-all", "{value}" }
        }
    }
}
