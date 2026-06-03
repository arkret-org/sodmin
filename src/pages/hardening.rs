//! T8.3 — aggregate production hardening dashboard.
//!
//! Every Cokret service (`soland`, `coauth`, `floria`, `starid`,
//! `teabay`) exposes a non-sensitive `hardening` block on its
//! `/health` endpoint. This page fans out across whichever services
//! the operator has wired (via `coauth_public_url` /
//! `starid_public_url` / the locally served principal), then renders a
//! per-service checklist with green / red chips and an aggregate
//! score across the whole deployment.
//!
//! Hardening status is intentionally coarse — no paths, hostnames, or
//! token tails are returned by any service — so it's safe to publish
//! on the unauthenticated `/health` route.

use dioxus::prelude::*;

use crate::api::server;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::page_header::PageHeader;
use crate::types::api::HardeningStatus;
use crate::utils::net::error::HttpError;

/// Build a sentinel `HttpError` for the "service not configured"
/// branch of the resource future. The dashboard treats this as a
/// `not_configured` empty state — distinct from an unreachable fetch.
fn http_skip(message: &str) -> HttpError {
    HttpError::message(message)
}

/// Aggregate view per service. `hardening` is `None` when the service
/// is reachable but the `/health` route did not return the new field
/// (older deployments), or `unreachable` is set when the fetch
/// failed.
struct ServiceHardening {
    /// Service slug used in URLs / DIDs (`soland`, `coauth`, `starid`,
    /// `floria`, `teabay`).
    slug: &'static str,
    /// User-facing label.
    label: &'static str,
    /// Short paragraph describing the service's role.
    description: &'static str,
    hardening: Option<HardeningStatus>,
    /// When `Some`, the operator has not wired this service.
    not_configured_label: Option<String>,
    /// When `true`, the fetch failed (network / 5xx).
    unreachable: bool,
}

#[component]
pub fn HardeningDashboard() -> Element {
    let mut soland_health = use_resource(|| async { server::get_soland_health().await });
    let mut coauth_health = use_resource(|| async {
        if !crate::utils::net::session::has_coauth() {
            return Err(http_skip("coauth not configured"));
        }
        server::get_coauth_health().await
    });
    let mut starid_health = use_resource(|| async {
        if !crate::utils::net::session::has_starid() {
            return Err(http_skip("starid not configured"));
        }
        server::get_starid_health().await
    });

    let services: Vec<ServiceHardening> = {
        let soland = soland_health.read();
        let coauth = coauth_health.read();
        let starid = starid_health.read();
        vec![
            service_from_health(
                "soland",
                "Principal server (soland)",
                "Move/Anchor/Lattice surface, admin auth, push bridge.",
                soland.as_ref(),
                None,
            ),
            service_from_health(
                "coauth",
                "Auth (coauth)",
                "OAuth/session issuer + admin identity provider.",
                coauth.as_ref(),
                if crate::utils::net::session::has_coauth() {
                    None
                } else {
                    Some("coauth public URL is not configured.".to_owned())
                },
            ),
            service_from_health(
                "starid",
                "Identity (starid)",
                "did:webvh writer + resolver.",
                starid.as_ref(),
                if crate::utils::net::session::has_starid() {
                    None
                } else {
                    Some("starid public URL is not configured.".to_owned())
                },
            ),
            // floria / teabay /health are not currently proxied through
            // sodmin. Surface them as informational entries so the
            // operator sees the full deployment surface and knows where
            // the next round of work is.
            ServiceHardening {
                slug: "floria",
                label: "Push gateway (floria)",
                description: "WebPush / APNs / FCM delivery worker.",
                hardening: None,
                not_configured_label: Some(
                    "floria `/health` not yet wired into sodmin; run the per-service checklist manually."
                        .to_owned(),
                ),
                unreachable: false,
            },
            ServiceHardening {
                slug: "teabay",
                label: "Directory (teabay)",
                description: "Applet / agent directory.",
                hardening: None,
                not_configured_label: Some(
                    "teabay `/health` not yet wired into sodmin; run the per-service checklist manually."
                        .to_owned(),
                ),
                unreachable: false,
            },
        ]
    };

    let (total_score, total_max) = aggregate_score(&services);
    let any_warnings = services
        .iter()
        .any(|s| s.hardening.as_ref().is_some_and(|h| !h.warnings.is_empty()));

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Hardening".to_string(),
                description: "Production deployment checklist aggregated across every Cokret service.".to_string(),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        soland_health.restart();
                        coauth_health.restart();
                        starid_health.restart();
                    },
                    "Refresh"
                }
            }

            // Overall score chip. Renders red when any warning is
            // active or score is below max; green otherwise.
            Card {
                CardContent { class: "p-4".to_string(),
                    div { class: "flex flex-col gap-1 sm:flex-row sm:items-center sm:justify-between",
                        div { class: "space-y-1",
                            p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                                "Aggregate score"
                            }
                            p { class: "text-2xl font-bold",
                                "{total_score} / {total_max}"
                            }
                            p { class: "text-xs text-muted-foreground",
                                "Sum of checklist scores across configured services. A clean production deployment scores at maximum."
                            }
                        }
                        {aggregate_badge(total_score, total_max, any_warnings)}
                    }
                }
            }

            // Per-service breakdown.
            div { class: "grid gap-4 lg:grid-cols-2",
                for service in services.iter() {
                    {service_card(service)}
                }
            }
        }
    }
}

fn service_from_health(
    slug: &'static str,
    label: &'static str,
    description: &'static str,
    fetched: Option<&Result<server::HealthEnvelope, crate::utils::net::error::HttpError>>,
    not_configured_label: Option<String>,
) -> ServiceHardening {
    let (hardening, unreachable) = match (fetched, &not_configured_label) {
        (_, Some(_)) => (None, false),
        (Some(Ok(env)), _) => (env.hardening.clone(), false),
        (Some(Err(_)), _) => (None, true),
        (None, _) => (None, false),
    };
    ServiceHardening {
        slug,
        label,
        description,
        hardening,
        not_configured_label,
        unreachable,
    }
}

fn aggregate_score(services: &[ServiceHardening]) -> (u32, u32) {
    services.iter().fold((0u32, 0u32), |(score, max), s| {
        if let Some(h) = s.hardening.as_ref() {
            (score + h.checklist_score, max + h.checklist_max)
        } else {
            (score, max)
        }
    })
}

fn aggregate_badge(score: u32, max: u32, any_warnings: bool) -> Element {
    if max == 0 {
        return rsx! {
            Badge { variant: BadgeVariant::Secondary, "No services reporting" }
        };
    }
    if score == max && !any_warnings {
        rsx! { Badge { variant: BadgeVariant::Success, "All checks passing" } }
    } else if score == 0 {
        rsx! { Badge { variant: BadgeVariant::Destructive, "All checks failing" } }
    } else {
        rsx! { Badge { variant: BadgeVariant::Destructive, "Some checks failing" } }
    }
}

fn service_card(service: &ServiceHardening) -> Element {
    let header_badge = if service.not_configured_label.is_some() {
        rsx! { Badge { variant: BadgeVariant::Secondary, "Not configured" } }
    } else if service.unreachable {
        rsx! { Badge { variant: BadgeVariant::Destructive, "Unreachable" } }
    } else if let Some(h) = service.hardening.as_ref() {
        if h.warnings.is_empty() {
            rsx! { Badge { variant: BadgeVariant::Success, "OK" } }
        } else {
            rsx! { Badge { variant: BadgeVariant::Destructive, "{h.warnings.len()} warning(s)" } }
        }
    } else {
        rsx! { Badge { variant: BadgeVariant::Secondary, "No data" } }
    };

    let body = if let Some(label) = service.not_configured_label.as_ref() {
        rsx! { p { class: "text-sm text-muted-foreground", "{label}" } }
    } else if service.unreachable {
        rsx! {
            p { class: "text-sm text-destructive",
                "Failed to fetch /health for this service. Check the upstream URL and that the service is reachable."
            }
        }
    } else if let Some(h) = service.hardening.as_ref() {
        render_checklist(h)
    } else {
        rsx! {
            p { class: "text-sm text-muted-foreground",
                "This service returned /health but did not include a hardening block. Upgrade the service to surface T8.3 status."
            }
        }
    };

    rsx! {
        Card {
            CardHeader {
                div { class: "flex items-start justify-between gap-2",
                    div { class: "space-y-1",
                        CardTitle { class: "text-lg".to_string(), "{service.label}" }
                        CardDescription { "{service.description}" }
                        p { class: "text-[10px] font-mono text-muted-foreground", "{service.slug}" }
                    }
                    {header_badge}
                }
            }
            CardContent { {body} }
        }
    }
}

fn render_checklist(h: &HardeningStatus) -> Element {
    let admin_auth = h.admin_auth_mode.clone().unwrap_or_else(|| "-".to_string());
    let rotation = h
        .provider_credential_rotation
        .clone()
        .unwrap_or_else(|| "-".to_string());

    // Per-check rows. Each row is a chip + label; chip turns red when
    // the corresponding flag is `false` (or `development_mode` is
    // true / admin_auth is `"development"` / `"closed"` / rotation is
    // `"none"`).
    let rows: Vec<(&str, bool)> = vec![
        ("development_mode disabled", !h.development_mode),
        ("TLS enabled", h.tls_enabled),
        ("CSP header configured", h.csp_header_configured),
        ("CORS strict", h.cors_strict),
        ("Secret manager in use", h.secret_manager_in_use),
        ("Log redaction enabled", h.log_redaction_enabled),
        (
            "Admin auth in production mode",
            !matches!(admin_auth.as_str(), "development" | "closed" | "-"),
        ),
        ("Rate limit enabled", h.rate_limit_enabled),
        (
            "Provider credential rotation",
            !matches!(rotation.as_str(), "none" | "-"),
        ),
    ];

    rsx! {
        div { class: "space-y-3",
            // Header score line.
            div { class: "flex items-center justify-between",
                p { class: "text-xs font-semibold uppercase tracking-wider text-muted-foreground",
                    "Score"
                }
                p { class: "text-sm font-mono",
                    "{h.checklist_score} / {h.checklist_max}"
                }
            }

            // Free-form chips listing each check.
            ul { class: "grid gap-1 sm:grid-cols-2",
                for (label, ok) in rows.iter() {
                    li {
                        class: if *ok {
                            "inline-flex items-center gap-1 rounded-md border border-green-600/30 bg-green-600/10 px-2 py-1 text-xs text-green-700 dark:text-green-300"
                        } else {
                            "inline-flex items-center gap-1 rounded-md border border-red-600/30 bg-red-600/10 px-2 py-1 text-xs text-red-700 dark:text-red-300"
                        },
                        span { class: "font-mono",
                            if *ok { "\u{2713}" } else { "\u{2717}" }
                        }
                        span { "{label}" }
                    }
                }
            }

            // Coarse strings (admin_auth_mode, rotation) shown beside
            // the chip row.
            div { class: "grid gap-2 sm:grid-cols-2 pt-1",
                div {
                    p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "admin_auth_mode" }
                    p { class: "text-xs font-mono", "{admin_auth}" }
                }
                div {
                    p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "provider_credential_rotation" }
                    p { class: "text-xs font-mono", "{rotation}" }
                }
            }

            // Warnings list.
            if !h.warnings.is_empty() {
                div { class: "rounded-md border border-red-600 bg-red-600/10 px-3 py-2 text-xs space-y-1",
                    role: "alert",
                    p { class: "font-semibold text-red-700 dark:text-red-300",
                        "Failing checks"
                    }
                    ul { class: "list-disc pl-5 text-red-700 dark:text-red-200",
                        for warning in h.warnings.iter() {
                            li { "{warning}" }
                        }
                    }
                }
            }
        }
    }
}
