use dioxus::prelude::*;

use crate::api::server;
use crate::components::ui::card::*;
use crate::components::ui::icons::Icon;
use crate::components::ui::loading::StatsSkeleton;
use crate::types::{ServerDescribeResponse, ServerStatusResponse};
use crate::utils::cache::{get_cached, set_cached};
use crate::utils::i18n::t;
use crate::utils::perf;

const CACHE_TTL_MS: f64 = 300_000.0;

#[component]
pub fn Dashboard() -> Element {
    let stats = use_resource(|| async {
        if let Some(cached) = get_cached("dashboard_stats", CACHE_TTL_MS) {
            return serde_json::from_str(&cached).ok();
        }
        match server::get_server_stats().await.ok() {
            Some(val) => {
                if let Ok(json) = serde_json::to_string(&val) {
                    set_cached("dashboard_stats", &json);
                }
                Some(val)
            }
            None => None,
        }
    });

    let server_info = use_resource(|| async {
        if let Some(cached) = get_cached("dashboard_server_info", CACHE_TTL_MS) {
            return serde_json::from_str(&cached).ok();
        }
        match server::get_server_info().await.ok() {
            Some(val) => {
                if let Ok(json) = serde_json::to_string(&val) {
                    set_cached("dashboard_server_info", &json);
                }
                Some(val)
            }
            None => None,
        }
    });

    let server_describe = use_resource(|| async {
        if let Some(cached) = get_cached("dashboard_server_describe", CACHE_TTL_MS) {
            return serde_json::from_str(&cached).ok();
        }
        match server::get_server_describe().await.ok() {
            Some(val) => {
                if let Ok(json) = serde_json::to_string(&val) {
                    set_cached("dashboard_server_describe", &json);
                }
                Some(val)
            }
            None => None,
        }
    });

    let coauth_describe = use_resource(|| async {
        if !crate::utils::session::has_coauth() {
            return None;
        }
        if let Some(cached) = get_cached("dashboard_coauth_describe", CACHE_TTL_MS) {
            return serde_json::from_str(&cached).ok();
        }
        match server::get_coauth_server_describe().await.ok() {
            Some(val) => {
                if let Ok(json) = serde_json::to_string(&val) {
                    set_cached("dashboard_coauth_describe", &json);
                }
                Some(val)
            }
            None => None,
        }
    });

    let server_status = use_resource(|| async {
        if let Some(cached) = get_cached("dashboard_server_status", CACHE_TTL_MS) {
            return serde_json::from_str(&cached).ok();
        }
        match server::get_server_status().await.ok() {
            Some(val) => {
                if let Ok(json) = serde_json::to_string(&val) {
                    set_cached("dashboard_server_status", &json);
                }
                Some(val)
            }
            None => None,
        }
    });

    let is_loading = stats.read().is_none();

    if is_loading {
        return rsx! {
            div { class: "space-y-6",
                div {
                    h1 { class: "text-2xl font-bold tracking-tight", {t("dashboard.title")} }
                    p { class: "text-muted-foreground", {t("dashboard.welcome")} }
                }
                StatsSkeleton { count: 6 }
            }
        };
    }

    let stats_data = stats.read().clone().flatten();
    let info_data = server_info.read().clone().flatten();
    let describe_data = server_describe.read().clone().flatten();
    let coauth_describe_data = coauth_describe.read().clone().flatten();
    let status_data = server_status.read().clone().flatten();

    let version_str = info_data
        .as_ref()
        .map(|i| i.server_version.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let protocol_str = describe_data
        .as_ref()
        .and_then(|i| i.protocol_version.clone())
        .or_else(|| info_data.as_ref().and_then(|i| i.protocol_version.clone()))
        .unwrap_or_else(|| "1.0".to_string());

    let service_did = describe_data
        .as_ref()
        .and_then(|d| non_empty(d.service_did.clone()))
        .unwrap_or_else(|| "-".to_string());
    let coauth_issuer_did = coauth_describe_data
        .as_ref()
        .and_then(coauth_issuer_did)
        .unwrap_or_else(|| "-".to_string());
    let delegated_resolver_endpoint = coauth_describe_data
        .as_ref()
        .and_then(identity_registry_endpoint)
        .unwrap_or_else(|| "-".to_string());
    let supported_profiles = describe_data
        .as_ref()
        .map(|d| join_or_dash(&d.supported_profiles))
        .unwrap_or_else(|| "-".to_string());
    let reducer_profile = describe_data
        .as_ref()
        .map(|d| join_or_dash(&d.supported_reducer_profiles))
        .unwrap_or_else(|| "-".to_string());
    let schema_registry = describe_data
        .as_ref()
        .and_then(schema_registry_version)
        .unwrap_or_else(|| "-".to_string());
    let event_kind_registry = describe_data
        .as_ref()
        .and_then(event_kind_registry_version)
        .unwrap_or_else(|| "-".to_string());
    let openapi_version = describe_data
        .as_ref()
        .and_then(|d| d.openapi_version.clone())
        .unwrap_or_else(|| "-".to_string());
    let conformance = describe_data
        .as_ref()
        .and_then(conformance_level)
        .unwrap_or_else(|| "Unknown".to_string());
    let health_state = health_summary(status_data.as_ref());
    let conformance_rows = conformance_rows(describe_data.as_ref());

    rsx! {
        div { class: "space-y-6",
            div {
                h1 { class: "text-2xl font-bold tracking-tight", {t("dashboard.title")} }
                p { class: "text-muted-foreground", {t("dashboard.welcome")} }
            }

            {
                let s = stats_data.as_ref();
                let actor_count = s.map(|s| s.actor_count.to_string()).unwrap_or_else(|| "-".to_string());
                let space_count = s.map(|s| s.space_count.to_string()).unwrap_or_else(|| "-".to_string());
                let report_count = s.map(|s| s.report_count.to_string()).unwrap_or_else(|| "-".to_string());
                let active_count = s.map(|s| s.active_actor_count.to_string()).unwrap_or_else(|| "-".to_string());
                let peer_count = s.map(|s| s.federation_peer_count.to_string()).unwrap_or_else(|| "-".to_string());
                let avg = perf::average_latency();
                let latency_val = if avg > 0.0 { format!("{:.0}ms", avg) } else { "-".to_string() };

                rsx! {
                    div { class: "grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-7 rounded-lg border glass-panel overflow-hidden",
                        {stat_cell("server", &version_str, t("server.version"), t("dashboard.server_online"), true)}
                        {stat_cell("users", &actor_count, t("nav.actors"), t("dashboard.total_actors"), false)}
                        {stat_cell("message-square", &space_count, t("nav.spaces"), t("dashboard.total_spaces"), false)}
                        {stat_cell("flag", &report_count, t("nav.reports"), t("dashboard.pending_reports"), false)}
                        {stat_cell("user-check", &active_count, t("dashboard.active_users"), t("dashboard.active_actors"), false)}
                        {stat_cell("globe", &peer_count, t("nav.federation"), t("dashboard.federation_peers"), false)}
                        {stat_cell("activity", &latency_val, t("dashboard.api_latency"), t("dashboard.avg_api_response"), false)}
                    }
                }
            }

            if let Some(ref info) = info_data {
                Card {
                    CardHeader {
                        CardTitle { class: "flex items-center gap-2".to_string(),
                            Icon { name: "server".to_string(), class: "h-5 w-5".to_string() }
                            {t("dashboard.server_info")}
                        }
                    }
                    CardContent {
                        div { class: "grid gap-4 md:grid-cols-3",
                            div {
                                p { class: "text-sm text-muted-foreground", {t("server.version")} }
                                p { class: "text-lg font-semibold", "{version_str}" }
                            }
                            div {
                                p { class: "text-sm text-muted-foreground", {t("dashboard.protocol_version")} }
                                p { class: "text-lg font-semibold", "{protocol_str}" }
                            }
                            if let Some(ref name) = info.server_name {
                                div {
                                    p { class: "text-sm text-muted-foreground", {t("dashboard.server_name")} }
                                    p { class: "text-lg font-semibold", "{name}" }
                                }
                            }
                        }
                    }
                }
            }

            Card {
                CardHeader {
                    CardTitle { class: "flex items-center gap-2".to_string(),
                        Icon { name: "server".to_string(), class: "h-5 w-5".to_string() }
                        {t("server.features")}
                    }
                    CardDescription {
                        "Contrix discovery from /api/v1/server/describe, with legacy admin info as fallback."
                    }
                }
                CardContent {
                    div { class: "grid gap-4 md:grid-cols-2 xl:grid-cols-4",
                        {metadata_cell("Principal Server DID", service_did)}
                        {metadata_cell("coauth issuer DID", coauth_issuer_did)}
                        {metadata_cell("delegated/public DID resolver", delegated_resolver_endpoint)}
                        {metadata_cell("Supported profiles", supported_profiles)}
                        {metadata_cell("Reducer profile", reducer_profile)}
                        {metadata_cell("Schema registry", schema_registry)}
                        {metadata_cell("Event-kind registry", event_kind_registry)}
                        {metadata_cell("OpenAPI version", openapi_version)}
                    }
                }
            }

            Card {
                CardHeader {
                    CardTitle { class: "flex items-center gap-2".to_string(),
                        Icon { name: "activity".to_string(), class: "h-5 w-5".to_string() }
                        "Conformance Status"
                    }
                    CardDescription {
                        "Declared surfaces are shown from service discovery; missing entries remain unavailable until the server advertises them."
                    }
                }
                CardContent {
                    div { class: "mb-4 grid gap-4 md:grid-cols-2",
                        {metadata_cell("Overall conformance", conformance)}
                        {metadata_cell("Health", health_state)}
                    }
                    div { class: "grid gap-3 md:grid-cols-2 xl:grid-cols-3",
                        for row in conformance_rows.iter() {
                            {conformance_cell(row)}
                        }
                    }
                }
            }
        }
    }
}

fn stat_cell(
    icon: &'static str,
    value: &str,
    title: String,
    subtitle: String,
    online: bool,
) -> Element {
    rsx! {
        div { class: "p-3 border-b border-r border-border/50 last:border-r-0",
            div { class: "flex items-center justify-between mb-1.5",
                span { class: "text-[11px] font-medium uppercase tracking-wider text-muted-foreground", "{title}" }
                Icon { name: icon.to_string(), class: "h-3.5 w-3.5 text-muted-foreground".to_string() }
            }
            div { class: "text-xl font-bold leading-none", "{value}" }
            if online {
                div { class: "flex items-center gap-1.5 mt-1.5",
                    span { class: "relative flex h-1.5 w-1.5",
                        span { class: "animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75" }
                        span { class: "relative inline-flex rounded-full h-1.5 w-1.5 bg-green-500" }
                    }
                    p { class: "text-[10px] text-green-600 dark:text-green-400 font-medium uppercase tracking-wide", "{subtitle}" }
                }
            } else {
                p { class: "text-[10px] text-muted-foreground mt-1.5 truncate", "{subtitle}" }
            }
        }
    }
}

struct ConformanceRow {
    label: &'static str,
    state: &'static str,
    detail: String,
}

fn metadata_cell(label: &'static str, value: String) -> Element {
    rsx! {
        div { class: "min-w-0 rounded-md border border-border/50 p-3",
            p { class: "text-xs font-medium uppercase text-muted-foreground", "{label}" }
            p { class: "mt-1 break-words text-sm font-semibold", "{value}" }
        }
    }
}

fn conformance_cell(row: &ConformanceRow) -> Element {
    let label = row.label;
    let state = row.state;
    let detail = row.detail.clone();

    rsx! {
        div { class: "min-w-0 rounded-md border border-border/50 p-3",
            div { class: "flex items-start justify-between gap-3",
                div { class: "min-w-0",
                    p { class: "text-sm font-medium", "{label}" }
                    p { class: "mt-1 text-xs text-muted-foreground break-words", "{detail}" }
                }
                {status_pill(state)}
            }
        }
    }
}

fn status_pill(state: &str) -> Element {
    let class = match state {
        "Declared" => "border-green-500/30 bg-green-500/10 text-green-700 dark:text-green-300",
        "Not declared" => "border-amber-500/30 bg-amber-500/10 text-amber-700 dark:text-amber-300",
        _ => "border-border bg-muted text-muted-foreground",
    };

    rsx! {
        span { class: "shrink-0 rounded-full border px-2 py-0.5 text-[11px] font-semibold {class}",
            "{state}"
        }
    }
}

fn non_empty(value: String) -> Option<String> {
    let value = value.trim().to_string();
    if value.is_empty() { None } else { Some(value) }
}

fn join_or_dash(items: &[String]) -> String {
    if items.is_empty() {
        "-".to_string()
    } else {
        items.join(", ")
    }
}

fn coauth_issuer_did(describe: &ServerDescribeResponse) -> Option<String> {
    describe
        .auth_metadata
        .as_ref()
        .and_then(|metadata| metadata.issuer_did.clone())
        .or_else(|| non_empty(describe.service_did.clone()))
}

fn identity_registry_endpoint(describe: &ServerDescribeResponse) -> Option<String> {
    describe
        .identity_registry_resolver
        .as_ref()
        .and_then(|resolver| resolver.delegated_resolver.as_ref())
        .and_then(|delegated| delegated.resolver.clone())
}

fn schema_registry_version(describe: &ServerDescribeResponse) -> Option<String> {
    describe.schema_registry_version.clone().or_else(|| {
        (!describe.supported_schema_profiles.is_empty())
            .then(|| join_or_dash(&describe.supported_schema_profiles))
    })
}

fn event_kind_registry_version(describe: &ServerDescribeResponse) -> Option<String> {
    describe
        .event_kind_registry_version
        .clone()
        .or_else(|| json_string(&describe.registry, &["event_kind_registry_version"]))
}

fn conformance_level(describe: &ServerDescribeResponse) -> Option<String> {
    json_string(&describe.limits, &["profile_status", "conformance"])
}

fn health_summary(status: Option<&ServerStatusResponse>) -> String {
    match status {
        Some(status) if status.ok => "Healthy".to_string(),
        Some(status) => {
            let failing = status
                .results
                .iter()
                .filter(|component| !component.ok)
                .count();
            if failing == 0 {
                "Issues detected".to_string()
            } else {
                format!("{failing} issue(s)")
            }
        }
        None => "Unknown".to_string(),
    }
}

fn conformance_rows(describe: Option<&ServerDescribeResponse>) -> Vec<ConformanceRow> {
    let checks = [
        (
            "core_event_store",
            &["events.", "cx.events", "event_store", "events_api_minimal"][..],
        ),
        ("chat_mvp", &["chat", "messages."][..]),
        ("kanban_mvp", &["kanban", "card.", "container."][..]),
        (
            "principal_server",
            &["principal_server", "soland_limited_server"][..],
        ),
        ("identity_registry", &["identity_registry"][..]),
        ("push_gateway", &["push."][..]),
    ];

    checks
        .iter()
        .map(|(label, needles)| {
            let state = match describe {
                Some(describe) if has_any_declared_surface(describe, needles) => "Declared",
                Some(_) => "Not declared",
                None => "Unknown",
            };
            ConformanceRow {
                label: *label,
                state,
                detail: needles.join(" | "),
            }
        })
        .collect()
}

fn has_any_declared_surface(describe: &ServerDescribeResponse, needles: &[&str]) -> bool {
    let implemented_surfaces = json_strings(
        &describe.limits,
        &["profile_status", "implemented_surfaces"],
    );
    describe
        .supported_profiles
        .iter()
        .chain(describe.supported_features.iter())
        .chain(describe.supported_operations.iter())
        .chain(describe.supported_reducer_profiles.iter())
        .chain(describe.supported_schema_profiles.iter())
        .chain(implemented_surfaces.iter())
        .any(|value| {
            let value = value.to_ascii_lowercase();
            needles.iter().any(|needle| value.contains(*needle))
        })
}

fn json_string(value: &serde_json::Value, path: &[&str]) -> Option<String> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_str().map(ToOwned::to_owned)
}

fn json_strings(value: &serde_json::Value, path: &[&str]) -> Vec<String> {
    let mut current = value;
    for key in path {
        match current.get(*key) {
            Some(next) => current = next,
            None => return Vec::new(),
        }
    }
    current
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToOwned::to_owned))
                .collect()
        })
        .unwrap_or_default()
}
