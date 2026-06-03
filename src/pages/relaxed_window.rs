//! Round R2/R3 — Relaxed ephemeral window slider (T09).
//!
//! `ck.schema.ephemeral_envelope.v1` enforces an absolute hard ceiling
//! of 300_000 ms (5 minutes) on `expires_at - sent_at`. Operators may
//! set a *softer* "relaxed window" below the hard ceiling so their
//! deployment intentionally drops ephemeral signals earlier; this page
//! is the slider for that setting.
//!
//! When the deployment is running an audit compliance profile
//! (`attested_audit.e2ee.v1` / `disclosed_audit.e2ee.v1`), the relaxed
//! profile toggle is grey-disabled with a tooltip explaining that
//! audited deployments may not relax the ephemeral window — the
//! audited profile pins it at the protocol hard ceiling.

use dioxus::prelude::*;

use crate::api::server;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::toast::{ToastVariant, show_toast};

/// Absolute hard ceiling on the ephemeral signal window. Mirrors the SDK
/// `EPHEMERAL_ABSOLUTE_HARD_CEILING_MS`.
pub const EPHEMERAL_HARD_CEILING_MS: u32 = 300_000;

/// Floor for the slider — sub-second windows aren't meaningful for the
/// presence / typing / receipt.read / call.signal kinds.
pub const RELAXED_WINDOW_FLOOR_MS: u32 = 1_000;

/// Recognised audit-compliance profile identifiers. When any of these
/// is active, the relaxed profile toggle is disabled.
const AUDIT_PROFILES: [&str; 2] = ["attested_audit.e2ee.v1", "disclosed_audit.e2ee.v1"];

#[component]
pub fn RelaxedWindowPage() -> Element {
    let mut relaxed_ms = use_signal(|| 60_000u32);
    let mut relaxed_enabled = use_signal(|| true);
    let mut hydrated = use_signal(|| false);
    let mut saving = use_signal(|| false);
    let mut setting_data = use_resource(|| async { server::get_relaxed_window().await });

    if !*hydrated.read()
        && let Some(Ok(setting)) = setting_data.read().as_ref()
    {
        relaxed_ms.set(clamp_relaxed_window(setting.window_ms));
        relaxed_enabled.set(setting.enabled);
        hydrated.set(true);
    }

    let active_profile = setting_data
        .read()
        .as_ref()
        .and_then(|result| result.as_ref().ok())
        .and_then(|setting| setting.active_profile.clone())
        .unwrap_or_else(|| "default".to_string());
    let audited = is_audit_profile(&active_profile);
    let value = *relaxed_ms.read();
    let enabled = *relaxed_enabled.read();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Relaxed ephemeral window".to_string(),
                description: "Soft per-deployment cap on `expires_at - sent_at` for `ck.presence` / `ck.typing` / `ck.receipt.read` / `ck.call.signal`. Round R2/R3 T09.".to_string(),
            }

            Card {
                CardHeader {
                    div { class: "flex items-start justify-between gap-2",
                        div { class: "space-y-1",
                            CardTitle { class: "text-lg".to_string(), "Active compliance profile" }
                            CardDescription { "Determines whether the relaxed profile toggle is available." }
                        }
                        if audited {
                            Badge { variant: BadgeVariant::Destructive, "audit profile active" }
                        } else {
                            Badge { variant: BadgeVariant::Secondary, "default profile" }
                        }
                    }
                }
                CardContent {
                    p { class: "font-mono text-sm break-all", "{active_profile}" }
                    if audited {
                        p { class: "mt-2 text-xs text-muted-foreground",
                            "Audit profiles pin the ephemeral window at the protocol hard ceiling (5 min). The relaxed profile toggle is disabled until the audit profile is retired."
                        }
                    }
                }
            }

            Card {
                CardHeader {
                    CardTitle { class: "text-lg".to_string(), "Relaxed window" }
                    CardDescription {
                        "Soft cap, in milliseconds. Hard ceiling: 300_000 ms (5 min)."
                    }
                }
                CardContent { class: "space-y-3".to_string(),
                    label { class: "flex items-start gap-2 text-sm",
                        input {
                            r#type: "checkbox",
                            class: "mt-1",
                            checked: enabled && !audited,
                            disabled: audited,
                            onchange: move |evt| {
                                if audited {
                                    return;
                                }
                                let parsed = evt.value().parse::<bool>().unwrap_or(!enabled);
                                relaxed_enabled.set(parsed);
                            },
                        }
                        span {
                            "Relaxed profile enabled"
                            if audited {
                                span {
                                    class: "ml-2 text-xs text-muted-foreground",
                                    title: "disabled: audit-compliance profile active",
                                    "(disabled \u{2014} audit profile active)"
                                }
                            }
                        }
                    }

                    div { class: "space-y-1",
                        label { class: "text-sm font-medium",
                            "Window (ms): "
                            span { class: "font-mono", "{value}" }
                        }
                        input {
                            r#type: "range",
                            class: "w-full",
                            min: "{RELAXED_WINDOW_FLOOR_MS}",
                            max: "{EPHEMERAL_HARD_CEILING_MS}",
                            step: "1000",
                            value: "{value}",
                            disabled: audited || !enabled,
                            oninput: move |evt| {
                                let raw = evt.value();
                                let parsed: u32 = raw.parse().unwrap_or(EPHEMERAL_HARD_CEILING_MS);
                                relaxed_ms.set(clamp_relaxed_window(parsed));
                            },
                        }
                        p { class: "text-xs text-muted-foreground",
                            "Range: {RELAXED_WINDOW_FLOOR_MS} ms (1 s) \u{2013} {EPHEMERAL_HARD_CEILING_MS} ms (5 min)"
                        }
                    }
                    div { class: "flex justify-end",
                        Button {
                            variant: ButtonVariant::Default,
                            disabled: audited || !enabled,
                            onclick: move |_| {
                                let v = clamp_relaxed_window(*relaxed_ms.read());
                                relaxed_ms.set(v);
                                saving.set(true);
                                spawn(async move {
                                    let body = server::UpdateRelaxedWindowRequest {
                                        enabled: true,
                                        window_ms: v,
                                    };
                                    match server::update_relaxed_window(&body).await {
                                        Ok(_) => {
                                            show_toast(
                                                &format!("Relaxed window set to {v} ms."),
                                                ToastVariant::Success,
                                            );
                                            hydrated.set(false);
                                            setting_data.restart();
                                        }
                                        Err(err) => show_toast(
                                            &format!("Relaxed window update failed: {err}"),
                                            ToastVariant::Error,
                                        ),
                                    }
                                    saving.set(false);
                                });
                            },
                            "Save"
                        }
                    }
                }
            }
        }
    }
}

/// True if the supplied profile id is one of the audit-compliance
/// profiles that pins the ephemeral window. Done as a helper so the UI
/// gating + the unit tests share a single source of truth.
pub fn is_audit_profile(profile_id: &str) -> bool {
    AUDIT_PROFILES.contains(&profile_id)
}

/// Clamp a candidate window value into `[FLOOR, HARD_CEILING]`. The
/// HTML `<input type="range">` already enforces this on the client,
/// but the slider can be driven via `oninput` with arbitrary values
/// from accessibility tooling — this is the canonical check.
pub fn clamp_relaxed_window(value: u32) -> u32 {
    value.clamp(RELAXED_WINDOW_FLOOR_MS, EPHEMERAL_HARD_CEILING_MS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_profile_detection_matches_spec_strings() {
        assert!(is_audit_profile("attested_audit.e2ee.v1"));
        assert!(is_audit_profile("disclosed_audit.e2ee.v1"));
        assert!(!is_audit_profile("default"));
        assert!(!is_audit_profile("attested_hardware"));
        assert!(!is_audit_profile(""));
    }

    #[test]
    fn clamp_caps_at_hard_ceiling() {
        // Anything above 300_000 must be capped — this is the
        // protocol's absolute hard ceiling for the ephemeral window.
        assert_eq!(
            clamp_relaxed_window(EPHEMERAL_HARD_CEILING_MS + 1),
            EPHEMERAL_HARD_CEILING_MS
        );
        assert_eq!(clamp_relaxed_window(10_000_000), EPHEMERAL_HARD_CEILING_MS);
    }

    #[test]
    fn clamp_floors_at_window_floor() {
        assert_eq!(clamp_relaxed_window(0), RELAXED_WINDOW_FLOOR_MS);
        assert_eq!(
            clamp_relaxed_window(RELAXED_WINDOW_FLOOR_MS - 1),
            RELAXED_WINDOW_FLOOR_MS
        );
    }

    #[test]
    fn clamp_passes_in_range_values_through() {
        assert_eq!(clamp_relaxed_window(60_000), 60_000);
        assert_eq!(
            clamp_relaxed_window(EPHEMERAL_HARD_CEILING_MS),
            EPHEMERAL_HARD_CEILING_MS
        );
        assert_eq!(
            clamp_relaxed_window(RELAXED_WINDOW_FLOOR_MS),
            RELAXED_WINDOW_FLOOR_MS
        );
    }
}
