use dioxus::prelude::*;

use crate::api::server;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::types::ServerDescribeOutcome;
use crate::utils::i18n::t;

#[component]
pub fn ServerStatus() -> Element {
    let mut info_data = use_resource(|| async { server::get_server_info().await.ok() });
    let mut status_data = use_resource(|| async { server::get_server_status().await.ok() });
    let mut admin_describe = use_resource(|| async { server::get_server_describe().await.ok() });
    let mut coauth_describe = use_resource(|| async {
        if !crate::utils::net::session::has_coauth() {
            return None;
        }
        server::get_coauth_server_describe().await.ok()
    });

    let info = info_data.read().clone().flatten();
    let status = status_data.read().clone().flatten();
    let admin_d = admin_describe.read().clone().flatten();
    let coauth_d = coauth_describe.read().clone().flatten();
    let coauth_configured = crate::utils::net::session::has_coauth();
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
    describe: Option<&ServerDescribeOutcome>,
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

fn describe_body(describe: &ServerDescribeOutcome) -> Element {
    let did = if describe.service_did.is_empty() {
        "-".to_string()
    } else {
        describe.service_did.clone()
    };
    // Round 4 — `trust_domain` is a required ServerDescribe v2 field;
    // an empty value means the server is pre-round-4 or misconfigured.
    let trust_domain = describe
        .trust_domain
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "-".to_string());
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
    let implemented = describe.implemented_features.clone();
    let operations = describe.supported_operations.clone();
    let bindings = describe.supported_bindings.clone();

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
    // Round 4 — when development_mode + verified_profiles both present,
    // the server is making contradictory claims. Surface a loud red
    // banner above the rest of the card.
    let dev_with_verified = describe.dev_mode_with_verified_profiles();

    let rate_limit_text = if describe.rate_limit.is_null() {
        "-".to_string()
    } else {
        serde_json::to_string_pretty(&describe.rate_limit)
            .unwrap_or_else(|_| describe.rate_limit.to_string())
    };
    let limits_text = if describe.limits.is_null() {
        "-".to_string()
    } else {
        serde_json::to_string_pretty(&describe.limits)
            .unwrap_or_else(|_| describe.limits.to_string())
    };

    rsx! {
        div { class: "space-y-4",
            // Round 4 — verified-profiles-in-dev-mode contradiction
            // banner. Renders ABOVE the dev-mode chip so operators can't
            // miss the most important wire-correctness issue.
            if dev_with_verified {
                div {
                    class: "rounded-md border-2 border-red-600 bg-red-600/15 px-3 py-2 text-sm font-semibold text-red-700 dark:text-red-200 space-y-1",
                    role: "alert",
                    p {
                        span { class: "mr-2", "\u{26A0}" }
                        {t("server_status.dev_with_verified_title")}
                    }
                    p { class: "text-xs font-normal",
                        {t("server_status.dev_with_verified_detail")}
                    }
                }
            }
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
                {info_cell(t("server_status.trust_domain"), trust_domain)}
                {info_cell(t("server_status.service_type"), service_type)}
                {info_cell(t("server_status.protocol_version"), protocol)}
                {info_cell(t("server_status.openapi_version"), openapi)}
                {info_cell(t("server_status.schema_registry"), schema)}
                {info_cell(t("server_status.event_kind_registry"), event_kind)}
                {info_cell(t("server_status.proof_verifier_mode"), proof_verifier_mode)}
                {info_cell(t("server_status.admin_auth_mode"), admin_auth_mode)}
            }

            // Round 4 — chip lists for the four list-shaped ServerDescribe
            // v2 fields: supported_profiles / supported_features /
            // implemented_features / supported_operations. Each renders
            // as a flat strip of mono chips; conformance buckets render
            // separately below.
            {chip_section(t("server_status.profiles_label"), &profiles)}
            {r3_profile_status_section(&profiles)}
            {chip_section(t("server_status.features_label"), &features)}
            {chip_section(t("server_status.implemented_features"), &implemented)}
            {chip_section(t("server_status.supported_operations"), &operations)}

            {binding_chip_section(t("server_status.supported_bindings"), &bindings)}

            // Round 4 — limits + rate_limit as raw JSON. Free-form per
            // SDK; UI cannot assume a fixed key set.
            div { class: "grid gap-3 sm:grid-cols-2",
                div { class: "space-y-1",
                    p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                        {t("server_status.limits_label")}
                    }
                    pre { class: "font-mono text-[11px] bg-muted/50 rounded p-2 overflow-x-auto",
                        "{limits_text}"
                    }
                }
                {rate_limit_panel(t("server_status.rate_limit_label"), &describe.rate_limit, rate_limit_text)}
            }

            // T6.2 §1 — conformance posture grouped into four buckets.
            // verified profiles flip to ❌ when development_mode is on,
            // since soland's relaxed proof verifier voids the
            // verification claim.
            {conformance_section(describe, development_mode_is_dev)}

            // T6.2 §6 — when dev posture is active, surface
            // plaintext_visibility in the same red card so the operator
            // sees every weak knob in one place.
            {dev_posture_card(describe)}
        }
    }
}

fn binding_chip_section(label: String, bindings: &[serde_json::Value]) -> Element {
    if bindings.is_empty() {
        return rsx! {
            div { class: "space-y-1",
                p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                    "{label}"
                }
                p { class: "text-xs text-muted-foreground", "-" }
            }
        };
    }

    rsx! {
        div { class: "space-y-1",
            p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                "{label}"
            }
            div { class: "flex flex-wrap gap-1",
                for binding in bindings.iter() {
                    {
                        let binding_type = binding
                            .get("type")
                            .or_else(|| binding.get("kind"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("binding");
                        let endpoint = binding
                            .get("endpoint")
                            .or_else(|| binding.get("url"))
                            .or_else(|| binding.get("service_endpoint"))
                            .and_then(|v| v.as_str());
                        let label = endpoint
                            .map(|e| format!("{binding_type}: {e}"))
                            .unwrap_or_else(|| binding_type.to_string());
                        rsx! {
                            Badge {
                                variant: BadgeVariant::Secondary,
                                class: "font-mono text-xs max-w-full truncate".to_string(),
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn rate_limit_panel(
    label: String,
    rate_limit: &serde_json::Value,
    fallback_text: String,
) -> Element {
    let bars = rate_limit_bars(rate_limit);

    rsx! {
        div { class: "space-y-2",
            p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                "{label}"
            }
            if bars.is_empty() {
                pre { class: "font-mono text-[11px] bg-muted/50 rounded p-2 overflow-x-auto",
                    "{fallback_text}"
                }
            } else {
                div { class: "space-y-2 rounded-md bg-muted/50 p-2",
                    for bar in bars.iter() {
                        {
                            let width = bar.percent.clamp(4, 100);
                            rsx! {
                                div { class: "space-y-1",
                                    div { class: "flex items-center justify-between gap-2 text-[11px]",
                                        span { class: "font-mono", "{bar.label}" }
                                        span { class: "text-muted-foreground", "{bar.value}" }
                                    }
                                    div {
                                        class: "h-2 overflow-hidden rounded-full bg-background",
                                        role: "meter",
                                        aria_valuemin: "0",
                                        aria_valuemax: "100",
                                        aria_valuenow: "{bar.percent}",
                                        aria_label: format!("rate limit {}", bar.label),
                                        div {
                                            class: "h-full rounded-full bg-primary",
                                            style: "width: {width}%;"
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
}

#[derive(Debug, Clone)]
struct RateLimitBar {
    label: String,
    value: String,
    percent: i32,
}

fn rate_limit_bars(rate_limit: &serde_json::Value) -> Vec<RateLimitBar> {
    let Some(obj) = rate_limit.as_object() else {
        return Vec::new();
    };
    if obj
        .get("disabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        return vec![RateLimitBar {
            label: "disabled".to_string(),
            value: "off".to_string(),
            percent: 100,
        }];
    }

    let keys = [
        "limit",
        "burst",
        "capacity",
        "tokens",
        "remaining",
        "requests",
    ];
    let mut bars = Vec::with_capacity(keys.len());
    for key in keys {
        if let Some(value) = obj.get(key).and_then(number_value) {
            let base = obj
                .get("limit")
                .and_then(number_value)
                .or_else(|| obj.get("capacity").and_then(number_value))
                .unwrap_or(value)
                .max(1.0);
            bars.push(RateLimitBar {
                label: key.to_string(),
                value: format_number(value),
                percent: ((value / base) * 100.0).round() as i32,
            });
        }
    }
    if let Some(window) = obj
        .get("window")
        .or_else(|| obj.get("window_seconds"))
        .or_else(|| obj.get("refill_interval_seconds"))
        .and_then(number_value)
    {
        bars.push(RateLimitBar {
            label: "window".to_string(),
            value: format!("{}s", format_number(window)),
            percent: 100,
        });
    }
    bars
}

fn number_value(value: &serde_json::Value) -> Option<f64> {
    value
        .as_f64()
        .or_else(|| value.as_str().and_then(|s| s.parse::<f64>().ok()))
}

fn format_number(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{}", value as u64)
    } else {
        format!("{value:.2}")
    }
}

/// Round 4 — uniform helper for the chip-list ServerDescribe v2 fields
/// (supported_profiles / supported_features / implemented_features /
/// supported_operations). Returns an empty fragment when the slice is
/// empty so we don't clutter the card with `-` rows.
fn chip_section(label: String, items: &[String]) -> Element {
    if items.is_empty() {
        return rsx! {
            div { class: "space-y-1",
                p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                    "{label}"
                }
                p { class: "text-xs text-muted-foreground", "-" }
            }
        };
    }
    rsx! {
        div { class: "space-y-1",
            p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                "{label}"
            }
            div { class: "flex flex-wrap gap-1",
                for item in items.iter() {
                    Badge { variant: BadgeVariant::Secondary, class: "font-mono text-xs".to_string(), "{item}" }
                }
            }
        }
    }
}

/// T6.2 §1 — render the four conformance buckets with their colour
/// codes. When `dev_mode_active` is true, verified profiles are crossed
/// out and labelled "unavailable in dev mode" because the relaxed
/// proof verifier breaks the verification chain.
fn conformance_section(describe: &ServerDescribeOutcome, dev_mode_active: bool) -> Element {
    let verified = describe
        .verified_profiles
        .iter()
        .map(|profile| profile.profile_id().to_owned())
        .collect::<Vec<_>>();
    let claimed = describe
        .claimed_profiles
        .iter()
        .map(|profile| profile.profile_id().to_owned())
        .collect::<Vec<_>>();
    let experimental = describe.experimental_features.clone();
    let compat = describe
        .compat_surfaces
        .iter()
        .map(|surface| surface.name().to_owned())
        .collect::<Vec<_>>();

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
fn dev_posture_card(describe: &ServerDescribeOutcome) -> Element {
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
    let plaintext = describe.plaintext_visibility.clone();

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

/// R3 (UI-6) — render a single-line status row per "new R3 profile" so
/// the operator can see at a glance which of them the server has
/// declared in `ck.server.describe.supported_profiles`. The list of
/// known R3 profiles is held here (not in i18n) because it tracks the
/// spec one-for-one and the i18n value is only the human label.
fn r3_profile_status_section(profiles: &[String]) -> Element {
    // (wire profile id, i18n key for the description copy).
    let known: &[(&str, &str)] = &[
        (
            "ck.profile.media_service_binding.v1",
            "server_status.profile.media_service_binding",
        ),
        (
            "ck.profile.accountable_principals.strict_reject.v1",
            "server_status.profile.accountable_principals_strict_reject",
        ),
        (
            "ck.profile.key_backup.memory_hard.v1",
            "server_status.profile.key_backup_memory_hard",
        ),
    ];

    rsx! {
        div { class: "space-y-1",
            p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                "R3 profile toggles"
            }
            ul { class: "text-xs space-y-1",
                for (wire, key) in known.iter() {
                    {
                        let declared = profiles.iter().any(|p| p == wire);
                        let copy = t(key);
                        rsx! {
                            li { class: "flex items-start gap-2",
                                if declared {
                                    span { class: "rounded bg-green-600/15 px-1.5 py-0.5 font-mono text-green-700 dark:text-green-300",
                                        "declared"
                                    }
                                } else {
                                    span { class: "rounded bg-muted px-1.5 py-0.5 font-mono text-muted-foreground",
                                        "absent"
                                    }
                                }
                                span { class: "font-mono", "{wire}" }
                                span { class: "text-muted-foreground", "— {copy}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
