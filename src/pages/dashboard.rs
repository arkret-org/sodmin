use dioxus::prelude::*;

use crate::api::server;
use crate::components::ui::card::*;
use crate::components::ui::icons::Icon;
use crate::components::ui::loading::StatsSkeleton;
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

    let version_str = info_data
        .as_ref()
        .map(|i| i.server_version.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let protocol_str = info_data
        .as_ref()
        .and_then(|i| i.protocol_version.clone())
        .unwrap_or_else(|| "1.0".to_string());

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
