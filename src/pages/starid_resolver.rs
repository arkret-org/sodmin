//! Read-only "Starid resolver status" panel (round C35.4).
//!
//! Mirrors the upstream starid resolver's `/_arkret/root/identity/describe`
//! envelope so the operator can see at a glance whether the writer is
//! healthy, what version of the did:webvh log is at the head, how many
//! witness attestations have been accepted, and how recently the log
//! moved.
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
    let head = describe
        .head_version_id
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "-".to_string());
    let witness = describe.witness_count.to_string();
    let freshness = describe
        .freshness
        .map(|ts| ts.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .unwrap_or_else(|| "-".to_string());
    let service_did = if describe.service_did.is_empty() {
        "-".to_string()
    } else {
        describe.service_did.clone()
    };
    let registry_mode = if describe.registry_mode.is_empty() {
        "-".to_string()
    } else {
        describe.registry_mode.clone()
    };
    let protocol = if describe.protocol_version.is_empty() {
        "-".to_string()
    } else {
        describe.protocol_version.clone()
    };

    let methods = describe.supported_methods.clone();
    let profiles = describe.profiles.clone();

    rsx! {
        Card {
            CardHeader {
                div { class: "flex items-center justify-between gap-2",
                    div { class: "space-y-1",
                        CardTitle { class: "text-lg".to_string(), {t("starid_resolver.card_title")} }
                        CardDescription { {t("starid_resolver.card_subtitle")} }
                    }
                    Badge {
                        variant: status_variant(&describe),
                        {status_label(&describe)}
                    }
                }
            }
            CardContent {
                div { class: "space-y-4",
                    div { class: "grid gap-3 sm:grid-cols-2 lg:grid-cols-3",
                        {info_cell(t("starid_resolver.service_did"), service_did)}
                        {info_cell(t("starid_resolver.registry_mode"), registry_mode)}
                        {info_cell(t("starid_resolver.protocol_version"), protocol)}
                        {info_cell(t("starid_resolver.head_version_id"), head)}
                        {info_cell(t("starid_resolver.witness_count"), witness)}
                        {info_cell(t("starid_resolver.freshness"), freshness)}
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

/// Status badge variant: a "warming up" registry that has never
/// witnessed activity (no head, no witnesses) reads as "Idle"; a head
/// with at least one witness reads as "Healthy"; everything in between
/// (head present, no witness yet) reads as "Pending witnesses".
fn status_variant(describe: &StaridDescribe) -> BadgeVariant {
    match status_class(describe) {
        StatusClass::Healthy => BadgeVariant::Success,
        StatusClass::PendingWitnesses => BadgeVariant::Outline,
        StatusClass::Idle => BadgeVariant::Secondary,
    }
}

fn status_label(describe: &StaridDescribe) -> String {
    match status_class(describe) {
        StatusClass::Healthy => t("starid_resolver.status_healthy"),
        StatusClass::PendingWitnesses => t("starid_resolver.status_pending"),
        StatusClass::Idle => t("starid_resolver.status_idle"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StatusClass {
    Healthy,
    PendingWitnesses,
    Idle,
}

fn status_class(describe: &StaridDescribe) -> StatusClass {
    match (describe.head_version_id.is_some(), describe.witness_count) {
        (true, n) if n > 0 => StatusClass::Healthy,
        (true, _) => StatusClass::PendingWitnesses,
        (false, _) => StatusClass::Idle,
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn empty_describe() -> StaridDescribe {
        StaridDescribe::default()
    }

    fn populated_describe() -> StaridDescribe {
        StaridDescribe {
            service_did: "did:web:starid.example".into(),
            registry_mode: "writer".into(),
            supported_methods: vec!["did:webvh".into(), "did:web".into()],
            supported_receipts: vec!["starid-local-sha256-v1".into()],
            protocol_version: "1.0".into(),
            profiles: vec!["ak.identity.webvh.v1".into()],
            head_version_id: Some("42-zABCDEF".into()),
            witness_count: 3,
            freshness: Some(
                chrono::Utc
                    .with_ymd_and_hms(2026, 5, 9, 10, 11, 12)
                    .unwrap(),
            ),
            hardening: None,
        }
    }

    #[test]
    fn empty_describe_classifies_as_idle() {
        let d = empty_describe();
        assert_eq!(status_class(&d), StatusClass::Idle);
    }

    #[test]
    fn head_without_witnesses_is_pending() {
        let mut d = empty_describe();
        d.head_version_id = Some("1-zSTART".into());
        assert_eq!(status_class(&d), StatusClass::PendingWitnesses);
    }

    #[test]
    fn head_with_witness_is_healthy() {
        let d = populated_describe();
        assert_eq!(status_class(&d), StatusClass::Healthy);
    }
}
