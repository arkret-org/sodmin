use dioxus::prelude::*;

use crate::api::server;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::types::ServerDescribeResBody;
use crate::utils::i18n::t;

#[component]
pub fn ServerStatus() -> Element {
    let mut info_data = use_resource(|| async { server::get_server_info().await.ok() });
    let mut status_data = use_resource(|| async { server::get_server_status().await.ok() });
    let mut admin_describe = use_resource(|| async { server::get_server_describe().await.ok() });
    let mut coauth_describe = use_resource(|| async {
        if !crate::utils::session::has_coauth() {
            return None;
        }
        server::get_coauth_server_describe().await.ok()
    });

    let info = info_data.read().clone().flatten();
    let status = status_data.read().clone().flatten();
    let admin_d = admin_describe.read().clone().flatten();
    let coauth_d = coauth_describe.read().clone().flatten();
    let coauth_configured = crate::utils::session::has_coauth();
    let status_resolved = status_data.read().is_some();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("server_status.title"),
                description: t("server_status.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        info_data.restart();
                        status_data.restart();
                        admin_describe.restart();
                        coauth_describe.restart();
                    },
                    {t("common.refresh")}
                }
            }

            if let Some(info) = info.as_ref() {
                {
                    let version = info.server_version.clone();
                    let protocol = info.protocol_version.clone().unwrap_or_else(|| "-".to_string());
                    let server_name = info.server_name.clone().unwrap_or_else(|| "-".to_string());
                    let uptime = info.uptime.map(|u| format!("{u}s")).unwrap_or_else(|| "-".to_string());
                    rsx! {
                        Card {
                            CardHeader { CardTitle { {t("server_status.server_info")} } }
                            CardContent {
                                div { class: "grid gap-4 md:grid-cols-4",
                                    {info_cell(t("server_status.version"), version)}
                                    {info_cell(t("server_status.protocol_version"), protocol)}
                                    {info_cell(t("server_status.server_name"), server_name)}
                                    {info_cell(t("server_status.uptime"), uptime)}
                                }
                            }
                        }
                    }
                }
            }

            // Multi-service describe board (G1).
            div { class: "space-y-4",
                h2 { class: "text-xl font-semibold tracking-tight", {t("server_status.services_title")} }
                div { class: "grid gap-4 lg:grid-cols-2",
                    {service_describe_card(
                        t("server_status.service_admin"),
                        t("server_status.service_admin_hint"),
                        admin_d.as_ref(),
                        None,
                    )}
                    {service_describe_card(
                        t("server_status.service_coauth"),
                        t("server_status.service_coauth_hint"),
                        coauth_d.as_ref(),
                        if coauth_configured {
                            None
                        } else {
                            Some(t("server_status.service_coauth_not_configured"))
                        },
                    )}
                    {service_describe_card(
                        t("server_status.service_soland"),
                        t("server_status.service_soland_hint"),
                        None,
                        Some(t("server_status.service_soland_not_configured")),
                    )}
                    {service_describe_card(
                        t("server_status.service_floria"),
                        t("server_status.service_floria_hint"),
                        None,
                        Some(t("server_status.service_floria_not_configured")),
                    )}
                }
            }

            // Component-level status panel (admin /server/status).
            div { class: "space-y-4",
                h2 { class: "text-xl font-semibold tracking-tight", {t("server_status.components_title")} }
                if let Some(status) = status.as_ref() {
                    div { class: "flex items-center gap-4 mb-4",
                        if status.ok {
                            Badge { variant: BadgeVariant::Success, class: "text-base px-4 py-1".to_string(), {t("server_status.healthy")} }
                        } else {
                            Badge { variant: BadgeVariant::Destructive, class: "text-base px-4 py-1".to_string(), {t("server_status.issues")} }
                        }
                    }
                    div { class: "grid gap-4 md:grid-cols-2 lg:grid-cols-3",
                        for component in status.results.iter() {
                            Card {
                                CardContent { class: "p-4".to_string(),
                                    div { class: "flex items-center justify-between",
                                        div { class: "space-y-1",
                                            p { class: "font-medium",
                                                {component.label.as_deref().unwrap_or("Unknown").to_string()}
                                            }
                                            if !component.ok {
                                                if let Some(ref reason) = component.reason {
                                                    p { class: "text-xs text-destructive mt-1", "{reason}" }
                                                }
                                            }
                                        }
                                        if component.ok {
                                            Badge { variant: BadgeVariant::Success, "OK" }
                                        } else {
                                            Badge { variant: BadgeVariant::Destructive, "Error" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if status_resolved {
                    Card {
                        CardContent { class: "p-8 text-center".to_string(),
                            p { class: "text-muted-foreground", {t("server_status.unable")} }
                        }
                    }
                } else {
                    PageSkeleton {}
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

fn service_describe_card(
    title: String,
    description: String,
    describe: Option<&ServerDescribeResBody>,
    not_configured_label: Option<String>,
) -> Element {
    let badge = match (describe.is_some(), not_configured_label.is_some()) {
        (true, _) => rsx! { Badge { variant: BadgeVariant::Success, {t("server_status.online")} } },
        (false, true) => {
            rsx! { Badge { variant: BadgeVariant::Secondary, {t("server_status.not_configured")} } }
        }
        (false, false) => {
            rsx! { Badge { variant: BadgeVariant::Destructive, {t("server_status.unreachable")} } }
        }
    };

    let body = match (describe, not_configured_label) {
        (Some(d), _) => describe_body(d),
        (None, Some(label)) => rsx! {
            p { class: "text-sm text-muted-foreground", "{label}" }
        },
        (None, None) => rsx! {
            p { class: "text-sm text-muted-foreground", {t("server_status.fetch_failed")} }
        },
    };

    rsx! {
        Card {
            CardHeader {
                div { class: "flex items-center justify-between gap-2",
                    div { class: "space-y-1",
                        CardTitle { class: "text-lg".to_string(), {title} }
                        CardDescription { {description} }
                    }
                    {badge}
                }
            }
            CardContent { {body} }
        }
    }
}

fn describe_body(describe: &ServerDescribeResBody) -> Element {
    let did = if describe.service_did.is_empty() {
        "-".to_string()
    } else {
        describe.service_did.clone()
    };
    let service_type = describe
        .service_type
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let protocol = describe
        .protocol_version
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let openapi = describe
        .openapi_version
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let schema = describe
        .schema_registry_version
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let event_kind = describe
        .event_kind_registry_version
        .clone()
        .unwrap_or_else(|| "-".to_string());

    let profiles = describe.supported_profiles.clone();
    let features = describe.supported_features.clone();

    // T1.4 — surface the runtime posture. `development_mode` is rendered
    // separately below (with red styling) so the operator can't miss it.
    let proof_verifier_mode = describe
        .proof_verifier_mode
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let admin_auth_mode = describe
        .admin_auth_mode
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let development_mode_label = match describe.development_mode {
        Some(true) => t("server_status.dev_banner"),
        Some(false) => "production".to_string(),
        None => "-".to_string(),
    };
    let development_mode_is_dev = describe.development_mode.unwrap_or(false);

    rsx! {
        div { class: "space-y-4",
            // Red posture chip — only renders when soland reports
            // `development_mode == true`. Production deployments see the
            // normal grid below with no extra chrome.
            if development_mode_is_dev {
                div {
                    class: "rounded-md border border-red-600 bg-red-600/10 px-3 py-2 text-sm font-semibold text-red-700 dark:text-red-300",
                    role: "alert",
                    span { class: "mr-2", "\u{26A0}" }
                    "{development_mode_label}"
                }
            }
            div { class: "grid gap-3 sm:grid-cols-2",
                {info_cell(t("server_status.service_did"), did)}
                {info_cell(t("server_status.service_type"), service_type)}
                {info_cell(t("server_status.protocol_version"), protocol)}
                {info_cell(t("server_status.openapi_version"), openapi)}
                {info_cell(t("server_status.schema_registry"), schema)}
                {info_cell(t("server_status.event_kind_registry"), event_kind)}
                {info_cell(t("server_status.proof_verifier_mode"), proof_verifier_mode)}
                {info_cell(t("server_status.admin_auth_mode"), admin_auth_mode)}
            }

            div { class: "space-y-1",
                p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                    {t("server_status.profiles_label")}
                }
                if profiles.is_empty() {
                    p { class: "text-xs text-muted-foreground", "-" }
                } else {
                    div { class: "flex flex-wrap gap-1",
                        for p in profiles.iter() {
                            Badge { variant: BadgeVariant::Secondary, class: "font-mono text-xs".to_string(), "{p}" }
                        }
                    }
                }
            }

            div { class: "space-y-1",
                p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                    {t("server_status.features_label")}
                }
                if features.is_empty() {
                    p { class: "text-xs text-muted-foreground", "-" }
                } else {
                    div { class: "flex flex-wrap gap-1",
                        for f in features.iter() {
                            Badge { variant: BadgeVariant::Secondary, class: "font-mono text-xs".to_string(), "{f}" }
                        }
                    }
                }
            }

            // T6.2 §1 — conformance posture grouped into four buckets.
            // verified profiles flip to ❌ when development_mode is on,
            // since soland's relaxed proof verifier voids the
            // verification claim.
            {conformance_section(describe, development_mode_is_dev)}

            // T6.2 §6 — when dev posture is active, surface
            // plaintext_visible_services in the same red card so the
            // operator sees every weak knob in one place.
            {dev_posture_card(describe)}
        }
    }
}

/// T6.2 §1 — render the four conformance buckets with their colour
/// codes. When `dev_mode_active` is true, verified profiles are crossed
/// out and labelled "unavailable in dev mode" because the relaxed
/// proof verifier breaks the verification chain.
fn conformance_section(describe: &ServerDescribeResBody, dev_mode_active: bool) -> Element {
    let verified = describe.verified_profiles.clone();
    let claimed = describe.claimed_profiles.clone();
    let experimental = describe.experimental_features.clone();
    let compat = describe.compat_surfaces.clone();

    if verified.is_empty() && claimed.is_empty() && experimental.is_empty() && compat.is_empty() {
        return rsx! {};
    }

    rsx! {
        div { class: "space-y-3 pt-2 border-t border-border/40",
            p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                {t("server_status.conformance_label")}
            }
            if !verified.is_empty() {
                {conformance_bucket(
                    t("server_status.verified_profiles"),
                    &verified,
                    if dev_mode_active { ConformanceTone::VerifiedBlockedByDev } else { ConformanceTone::Verified },
                )}
            }
            if !claimed.is_empty() {
                {conformance_bucket(
                    t("server_status.claimed_profiles"),
                    &claimed,
                    ConformanceTone::Claimed,
                )}
            }
            if !experimental.is_empty() {
                {conformance_bucket(
                    t("server_status.experimental_features"),
                    &experimental,
                    ConformanceTone::Experimental,
                )}
            }
            if !compat.is_empty() {
                {conformance_bucket(
                    t("server_status.compat_surfaces"),
                    &compat,
                    ConformanceTone::Compat,
                )}
            }
        }
    }
}

#[derive(Copy, Clone)]
enum ConformanceTone {
    Verified,
    VerifiedBlockedByDev,
    Claimed,
    Experimental,
    Compat,
}

fn conformance_bucket(label: String, items: &[String], tone: ConformanceTone) -> Element {
    let (chip_class, prefix, suffix_label) = match tone {
        ConformanceTone::Verified => (
            "bg-green-600/10 text-green-700 dark:text-green-300 border border-green-600/30",
            "\u{2713}",
            None,
        ),
        ConformanceTone::VerifiedBlockedByDev => (
            "bg-red-600/10 text-red-700 dark:text-red-300 border border-red-600/30 line-through",
            "\u{274C}",
            Some(t("server_status.verified_blocked_dev")),
        ),
        ConformanceTone::Claimed => (
            "bg-yellow-600/10 text-yellow-700 dark:text-yellow-300 border border-yellow-600/30",
            "\u{1F7E1}",
            None,
        ),
        ConformanceTone::Experimental => (
            "bg-blue-600/10 text-blue-700 dark:text-blue-300 border border-blue-600/30",
            "\u{1F9EA}",
            Some(t("server_status.experimental_warning")),
        ),
        ConformanceTone::Compat => (
            "bg-gray-500/10 text-gray-700 dark:text-gray-300 border border-gray-500/30",
            "\u{1F50C}",
            None,
        ),
    };

    rsx! {
        div { class: "space-y-1",
            p { class: "text-[11px] font-medium text-muted-foreground", "{label}" }
            div { class: "flex flex-wrap gap-1",
                for item in items.iter() {
                    span {
                        class: "inline-flex items-center gap-1 rounded-md px-2 py-0.5 font-mono text-xs {chip_class}",
                        span { "{prefix}" }
                        span { "{item}" }
                    }
                }
            }
            if let Some(suffix) = suffix_label.as_ref() {
                p { class: "text-[11px] italic text-muted-foreground", "{suffix}" }
            }
        }
    }
}

/// T6.2 §6 — surface every weak runtime knob as a red posture card.
/// Renders nothing on a clean production server.
fn dev_posture_card(describe: &ServerDescribeResBody) -> Element {
    let verifier_dev = describe
        .proof_verifier_mode
        .as_deref()
        .map(|m| m.eq_ignore_ascii_case("development"))
        .unwrap_or(false);
    let admin_dev = describe
        .admin_auth_mode
        .as_deref()
        .map(|m| m.eq_ignore_ascii_case("development"))
        .unwrap_or(false);
    let plaintext = describe.plaintext_visible_services.clone();

    if !verifier_dev && !admin_dev && plaintext.is_empty() {
        return rsx! {};
    }

    rsx! {
        div {
            class: "rounded-md border border-red-600 bg-red-600/10 dark:bg-red-600/20 px-3 py-2 text-xs space-y-1",
            role: "alert",
            p { class: "font-semibold text-red-700 dark:text-red-300",
                {t("server_status.dev_posture_title")}
            }
            ul { class: "list-disc pl-5 text-red-700 dark:text-red-200",
                if verifier_dev {
                    li { {t("server_status.dev_posture_verifier")} }
                }
                if admin_dev {
                    li { {t("server_status.dev_posture_admin")} }
                }
                if !plaintext.is_empty() {
                    li {
                        {format!("{}: {}", t("server_status.dev_posture_plaintext"), plaintext.join(", "))}
                    }
                }
            }
        }
    }
}
