use dioxus::prelude::*;

use crate::api::server;
use crate::components::dev_mode_banner::DevModeDashboardNotice;
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::icons::Icon;
use crate::components::ui::loading::StatsSkeleton;
use crate::types::{ServerDescribeDocument, ServerStatusOutcome};
use crate::utils::cache::cached_result;
use crate::utils::i18n::t;
use crate::utils::net::error::HttpError;
use crate::utils::net::perf;

const CACHE_TTL_MS: f64 = 300_000.0;

#[component]
pub fn Dashboard() -> Element {
    let stats = use_resource(|| async {
        cached_result("dashboard_stats", CACHE_TTL_MS, || async {
            server::get_server_stats().await
        })
        .await
    });

    let server_info = use_resource(|| async {
        cached_result("dashboard_server_info", CACHE_TTL_MS, || async {
            server::get_server_info().await
        })
        .await
    });

    let server_describe = use_resource(|| async {
        cached_result("dashboard_server_describe", CACHE_TTL_MS, || async {
            server::get_server_describe().await
        })
        .await
    });

    let coauth_describe = use_resource(|| async {
        if !crate::utils::net::session::has_coauth() {
            return None;
        }
        Some(
            cached_result("dashboard_coauth_describe", CACHE_TTL_MS, || async {
                server::get_coauth_server_describe().await
            })
            .await,
        )
    });

    let server_status = use_resource(|| async {
        cached_result("dashboard_server_status", CACHE_TTL_MS, || async {
            server::get_server_status().await
        })
        .await
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

    let stats_result = stats.read().clone();
    let info_result = server_info.read().clone();
    let describe_result = server_describe.read().clone();
    let coauth_describe_result = coauth_describe.read().clone().flatten();
    let status_result = server_status.read().clone();

    let stats_data = result_data(&stats_result).cloned();
    let info_data = result_data(&info_result).cloned();
    let describe_data = result_data(&describe_result).cloned();
    let coauth_describe_data = result_data(&coauth_describe_result).cloned();
    let status_data = result_data(&status_result).cloned();

    let version_str = info_data
        .as_ref()
        .map(|i| i.server_version.clone())
        .unwrap_or_else(|| t("common.unknown"));

    let protocol_str = describe_data
        .as_ref()
        .map(|i| i.protocol_version.clone())
        .or_else(|| info_data.as_ref().and_then(|i| i.protocol_version.clone()))
        .unwrap_or_else(|| "1.0".to_string());

    let service_did = describe_data
        .as_ref()
        .map(|d| d.service_did.to_string())
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
        .and_then(|d| d.extra_str(&["openapi_version"]))
        .unwrap_or_else(|| "-".to_string());
    let conformance = describe_data
        .as_ref()
        .and_then(conformance_level)
        .unwrap_or_else(|| t("common.unknown"));
    let health_state = health_summary(status_data.as_ref());
    let conformance_rows = conformance_rows(describe_data.as_ref());

    rsx! {
        div { class: "space-y-6",
            // T1.4: in-page red notice when the connected server reports
            // `development_mode == true`. Complements the top-of-page
            // banner so the warning is visible even after the operator
            // scrolls past the header.
            DevModeDashboardNotice {}
            div {
                h1 { class: "text-2xl font-bold tracking-tight", {t("dashboard.title")} }
                p { class: "text-muted-foreground", {t("dashboard.welcome")} }
            }

            {resource_error(&stats_result)}
            {resource_error(&info_result)}
            {resource_error(&describe_result)}
            {resource_error(&coauth_describe_result)}
            {resource_error(&status_result)}

            {
                let s = stats_data.as_ref();
                let actor_count = s.map(|s| s.actor_count.to_string()).unwrap_or_else(|| "-".to_string());
                let realm_count = s.map(|s| s.realm_count.to_string()).unwrap_or_else(|| "-".to_string());
                let active_count = s.map(|s| s.active_actor_count.to_string()).unwrap_or_else(|| "-".to_string());
                let peer_count = s.map(|s| s.federation_peer_count.to_string()).unwrap_or_else(|| "-".to_string());
                let avg = perf::average_latency();
                let latency_val = if avg > 0.0 { format!("{:.0}ms", avg) } else { "-".to_string() };

                rsx! {
                    div { class: "grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-6 rounded-lg border glass-panel overflow-hidden",
                        {stat_cell("server", &version_str, t("server.version"), t("dashboard.server_online"), true)}
                        {stat_cell("users", &actor_count, t("nav.actors"), t("dashboard.total_actors"), false)}
                        {stat_cell("shield", &realm_count, t("nav.realms"), t("dashboard.total_realms"), false)}
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
                        {t("dashboard.discovery_source")}
                    }
                }
                CardContent {
                    div { class: "grid gap-4 md:grid-cols-2 xl:grid-cols-4",
                        {metadata_cell(t("dashboard.principal_server_did"), service_did)}
                        {metadata_cell(t("dashboard.coauth_issuer_did"), coauth_issuer_did)}
                        {metadata_cell(t("dashboard.delegated_resolver"), delegated_resolver_endpoint)}
                        {metadata_cell(t("dashboard.supported_profiles"), supported_profiles)}
                        {metadata_cell(t("dashboard.reducer_profile"), reducer_profile)}
                        {metadata_cell(t("dashboard.schema_registry"), schema_registry)}
                        {metadata_cell(t("dashboard.event_kind_registry"), event_kind_registry)}
                        {metadata_cell(t("dashboard.openapi_version"), openapi_version)}
                    }
                }
            }

            Card {
                CardHeader {
                    CardTitle { class: "flex items-center gap-2".to_string(),
                        Icon { name: "activity".to_string(), class: "h-5 w-5".to_string() }
                        {t("dashboard.conformance_status")}
                    }
                    CardDescription {
                        {t("dashboard.conformance_desc")}
                    }
                }
                CardContent {
                    div { class: "mb-4 grid gap-4 md:grid-cols-2",
                        {metadata_cell(t("dashboard.overall_conformance"), conformance)}
                        {metadata_cell(t("dashboard.health"), health_state)}
                    }
                    div { class: "grid gap-3 md:grid-cols-2 xl:grid-cols-3",
                        for row in conformance_rows.iter() {
                            {conformance_cell(row)}
                        }
                    }
                    // Review D14 — the per-surface Declared/Not-declared states
                    // come from `has_any_declared_surface` substring matching,
                    // not an authoritative conformance field. Label them as a
                    // heuristic so operators do not treat them as ground truth.
                    p {
                        class: "mt-3 text-xs text-amber-700 dark:text-amber-300",
                        "data-testid": "conformance-heuristic-note",
                        span { class: "mr-1", "\u{2139}" }
                        {t("dashboard.conformance_heuristic_note")}
                    }
                }
            }
        }
    }
}

fn result_data<T>(result: &Option<Result<T, HttpError>>) -> Option<&T> {
    result.as_ref().and_then(|value| value.as_ref().ok())
}

fn resource_error<T>(result: &Option<Result<T, HttpError>>) -> Element {
    match result.as_ref().and_then(|value| value.as_ref().err()) {
        Some(error) => rsx! {
            ErrorBanner {
                message: error.message.clone(),
                errcode: error.body.as_ref().map(|body| body.errcode.clone()),
                request_id: error.request_id.clone(),
                retry_after_ms: error.retry_after_ms,
            }
        },
        None => rsx! {},
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

fn metadata_cell(label: String, value: String) -> Element {
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
    let display = match state {
        "Declared" => t("dashboard.declared"),
        "Not declared" => t("dashboard.not_declared"),
        _ => t("common.unknown"),
    };

    rsx! {
        span { class: "shrink-0 rounded-full border px-2 py-0.5 text-[11px] font-semibold {class}",
            "{display}"
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

fn coauth_issuer_did(describe: &ServerDescribeDocument) -> Option<String> {
    // `auth_metadata` is the SDK-canonical `AuthMetadata`; coauth carries its
    // issuer DID in the type's flattened `extra` (`additionalProperties`), so
    // serialize to JSON before walking for the `issuer_did` key.
    let auth_metadata = serde_json::to_value(&describe.auth_metadata).unwrap_or_default();
    json_string(&auth_metadata, &["issuer_did"])
        .or_else(|| non_empty(describe.service_did.to_string()))
}

fn identity_registry_endpoint(describe: &ServerDescribeDocument) -> Option<String> {
    // coauth-proprietary top-level extension block (not part of the spec
    // ServiceDescribe shape) — read via the extension envelope.
    describe.extra_str(&[
        "identity_registry_resolver",
        "delegated_resolver",
        "resolver",
    ])
}

fn schema_registry_version(describe: &ServerDescribeDocument) -> Option<String> {
    describe
        .extra_str(&["schema_registry_version"])
        .or_else(|| {
            (!describe.supported_schema_profiles.is_empty())
                .then(|| join_or_dash(&describe.supported_schema_profiles))
        })
}

fn event_kind_registry_version(describe: &ServerDescribeDocument) -> Option<String> {
    describe
        .extra_str(&["event_kind_registry_version"])
        .or_else(|| describe.extra_str(&["registry", "event_kind_registry_version"]))
}

fn conformance_level(describe: &ServerDescribeDocument) -> Option<String> {
    json_string(&describe.limits, &["profile_status", "conformance"])
}

fn health_summary(status: Option<&ServerStatusOutcome>) -> String {
    match status {
        Some(status) if status.ok => t("dashboard.health_healthy"),
        Some(status) => {
            let failing = status
                .results
                .iter()
                .filter(|component| !component.ok)
                .count();
            if failing == 0 {
                t("dashboard.health_issues_detected")
            } else {
                format!("{failing} {}", t("dashboard.health_issues_suffix"))
            }
        }
        None => t("common.unknown"),
    }
}

fn conformance_rows(describe: Option<&ServerDescribeDocument>) -> Vec<ConformanceRow> {
    let checks = [
        (
            "core_event_store",
            &["events.", "ck.events", "event_store", "events_api_minimal"][..],
        ),
        ("chat_mvp", &["chat", "messages."][..]),
        ("kanban_mvp", &["kanban", "card.", "container."][..]),
        ("principal_server", &["principal_server"][..]),
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
                label,
                state,
                detail: needles.join(" | "),
            }
        })
        .collect()
}

fn has_any_declared_surface(describe: &ServerDescribeDocument, needles: &[&str]) -> bool {
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
