//! Round R2/R3 — 7-domain deactivation fanout panel (T07).
//!
//! When an admin runs `ak.self.agent.deactivate` (or `ak.realm.destroy`),
//! the principal server cascades the deactivation across seven
//! independent local domains. This panel renders the per-domain result
//! so the operator can spot a partial fanout and retry the failing
//! domains.
//!
//! The seven domains, in the order the reducer fans out:
//!
//! 1. `session`       — coauth session revocation
//! 2. `device`        — MLS leaf removal + device tombstones
//! 3. `applet`        — installed-applet revocation (teabay)
//! 4. `keypackage`    — MLS keypackage withdrawal
//! 5. `push`          — push-route purge (floria)
//! 6. `to_device`     — queued to-device messages drained
//! 7. `capability`    — capability-cache invalidation
//!
//! For `ak.realm.destroy` the same panel is reused; in addition the
//! caller wires an "erasure receipt" cross-PS bar.

use dioxus::prelude::*;

use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::*;
use crate::utils::fmt::date::format_optional_iso_datetime;
use crate::utils::i18n::t;

/// Stable identifier for each of the seven local fanout domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FanoutDomain {
    Session,
    Device,
    Applet,
    Keypackage,
    Push,
    ToDevice,
    Capability,
}

impl FanoutDomain {
    pub fn slug(&self) -> &'static str {
        match self {
            FanoutDomain::Session => "session",
            FanoutDomain::Device => "device",
            FanoutDomain::Applet => "applet",
            FanoutDomain::Keypackage => "keypackage",
            FanoutDomain::Push => "push",
            FanoutDomain::ToDevice => "to_device",
            FanoutDomain::Capability => "capability",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            FanoutDomain::Session => "Sessions revoked",
            FanoutDomain::Device => "Devices tombstoned",
            FanoutDomain::Applet => "Applets revoked",
            FanoutDomain::Keypackage => "MLS keypackages withdrawn",
            FanoutDomain::Push => "Push routes purged",
            FanoutDomain::ToDevice => "To-device queue drained",
            FanoutDomain::Capability => "Capability cache invalidated",
        }
    }

    /// Canonical ordering used for rendering and for the "all done"
    /// progress percentage.
    pub fn all() -> [FanoutDomain; 7] {
        [
            FanoutDomain::Session,
            FanoutDomain::Device,
            FanoutDomain::Applet,
            FanoutDomain::Keypackage,
            FanoutDomain::Push,
            FanoutDomain::ToDevice,
            FanoutDomain::Capability,
        ]
    }
}

/// Outcome of a single fanout domain. Mirrors the principal-server
/// describe `{state, error_code?, attempt_count}` triple.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FanoutDomainResult {
    pub domain: FanoutDomain,
    pub state: FanoutState,
    /// Last error code (if state == Failed) — a bare registry code
    /// (e.g. `fanout_partial`).
    pub error_code: Option<String>,
    pub attempt_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanoutState {
    Succeeded,
    Failed,
}

impl FanoutState {
    pub fn badge_variant(&self) -> BadgeVariant {
        match self {
            FanoutState::Succeeded => BadgeVariant::Success,
            FanoutState::Failed => BadgeVariant::Destructive,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            FanoutState::Succeeded => "ok",
            FanoutState::Failed => "failed",
        }
    }
}

/// Top-level snapshot of all seven domains for one
/// `ak.self.agent.deactivate` (or `ak.realm.destroy`) invocation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FanoutSnapshot {
    pub subject_id: String,
    pub domains: Vec<FanoutDomainResult>,
    /// Only present on `ak.realm.destroy`. For `ak.self.agent.deactivate`
    /// this is `None`.
    pub erasure_receipt: Option<ErasureReceiptStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErasureReceiptStatus {
    pub receipt_id: String,
    pub local_state: FanoutState,
    /// Cross-PS aggregate state when the principal server exposes it.
    pub cross_ps_state: Option<FanoutState>,
    /// Per-peer receipt acknowledgements. Empty means the backend did
    /// not include peer-level fanout evidence for this invocation.
    pub cross_ps_peers: Vec<CrossPsFanoutResult>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrossPsFanoutResult {
    pub service_did: String,
    pub state: FanoutState,
    pub error_code: Option<String>,
    pub observed_at: Option<String>,
}

impl FanoutSnapshot {
    /// Convenience for unit tests + the dashboard chip — `(done, total)`
    /// across the seven domains.
    pub fn progress(&self) -> (usize, usize) {
        let total = FanoutDomain::all().len();
        let done = self
            .domains
            .iter()
            .filter(|d| matches!(d.state, FanoutState::Succeeded))
            .count();
        (done, total)
    }

    pub fn any_failed(&self) -> bool {
        self.domains
            .iter()
            .any(|d| matches!(d.state, FanoutState::Failed))
    }
}

/// Stub a snapshot for the placeholder dashboard view used while the
/// admin describe surface is being plumbed. Mirrors a realistic partial
/// fanout (push + to_device failed, retryable).
pub fn placeholder_snapshot(subject_id: impl Into<String>) -> FanoutSnapshot {
    let domains = FanoutDomain::all()
        .iter()
        .enumerate()
        .map(|(i, d)| FanoutDomainResult {
            domain: *d,
            state: match *d {
                FanoutDomain::Push | FanoutDomain::ToDevice => FanoutState::Failed,
                _ => FanoutState::Succeeded,
            },
            error_code: match *d {
                FanoutDomain::Push => Some("fanout_partial".to_string()),
                FanoutDomain::ToDevice => Some("fanout_partial".to_string()),
                _ => None,
            },
            attempt_count: (i as u32) + 1,
        })
        .collect();
    FanoutSnapshot {
        subject_id: subject_id.into(),
        domains,
        erasure_receipt: None,
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct DeactivationFanoutPanelProps {
    pub snapshot: FanoutSnapshot,
    /// Optional "Retry this domain" handler. The parent typically wires
    /// this to `POST /_soland/admin/identity/deactivations/{id}/retry`
    /// with the failing slug.
    #[props(default)]
    pub on_retry: EventHandler<FanoutDomain>,
}

#[component]
pub fn DeactivationFanoutPanel(props: DeactivationFanoutPanelProps) -> Element {
    let snapshot = props.snapshot.clone();
    let (done, total) = snapshot.progress();
    let any_failed = snapshot.any_failed();
    let receipt = snapshot.erasure_receipt.clone();
    let subject = snapshot.subject_id.clone();

    rsx! {
        Card {
            CardHeader {
                div { class: "flex items-start justify-between gap-2",
                    div { class: "space-y-1",
                        CardTitle { class: "text-lg".to_string(), "7-domain fanout" }
                        CardDescription { "subject: {subject}" }
                    }
                    if any_failed {
                        Badge { variant: BadgeVariant::Destructive, "Partial ({done}/{total})" }
                    } else if done == total {
                        Badge { variant: BadgeVariant::Success, "Complete ({done}/{total})" }
                    } else {
                        Badge { variant: BadgeVariant::Secondary, "In progress ({done}/{total})" }
                    }
                }
            }
            CardContent { class: "space-y-3".to_string(),
                // Per-domain rows in canonical order.
                ul { class: "space-y-2",
                    for result in snapshot.domains.iter() {
                        {
                            let domain = result.domain;
                            let slug = domain.slug();
                            let label = domain.label();
                            let state = result.state;
                            let state_label = state.label();
                            let state_variant = state.badge_variant();
                            let attempt = result.attempt_count;
                            let error = result.error_code.clone();
                            let row_class = if matches!(state, FanoutState::Failed) {
                                "rounded-md border border-red-600/40 bg-red-600/10 p-2 text-xs"
                            } else {
                                "rounded-md border p-2 text-xs"
                            };
                            rsx! {
                                li { class: "{row_class}",
                                    div { class: "flex items-center justify-between gap-2",
                                        div {
                                            div { class: "font-semibold", "{label}" }
                                            div { class: "font-mono text-[10px] text-muted-foreground",
                                                "{slug} \u{00b7} attempt={attempt}"
                                            }
                                            if let Some(code) = error.as_ref() {
                                                div { class: "font-mono text-[10px] text-destructive", "error_code={code}" }
                                            }
                                        }
                                        div { class: "flex items-center gap-2",
                                            Badge { variant: state_variant, "{state_label}" }
                                            if matches!(state, FanoutState::Failed) {
                                                {
                                                    let on_retry = props.on_retry;
                                                    rsx! {
                                                        Button {
                                                            variant: ButtonVariant::Outline,
                                                            size: ButtonSize::Sm,
                                                            onclick: move |_| on_retry.call(domain),
                                                            "Retry"
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

                // Erasure receipt — only present for `ak.realm.destroy`.
                if let Some(r) = receipt {
                    {erasure_receipt_block(&r)}
                }
            }
        }
    }
}

fn erasure_receipt_block(receipt: &ErasureReceiptStatus) -> Element {
    let local_variant = receipt.local_state.badge_variant();
    let local_label = receipt.local_state.label();
    let cross_ps_label = receipt
        .cross_ps_state
        .map(|s| s.label().to_string())
        .unwrap_or_else(|| "not reported".to_string());
    let cross_ps_variant = receipt
        .cross_ps_state
        .map(|s| s.badge_variant())
        .unwrap_or(BadgeVariant::Secondary);
    let total = receipt.cross_ps_peers.len();
    let done = receipt
        .cross_ps_peers
        .iter()
        .filter(|peer| matches!(peer.state, FanoutState::Succeeded))
        .count();
    let progress = (done * 100).checked_div(total).unwrap_or(0);

    rsx! {
        div { class: "rounded-md border p-3 space-y-2",
            div { class: "flex items-center justify-between gap-2",
                p { class: "text-sm font-semibold", "Erasure receipt" }
                div { class: "flex flex-wrap items-center gap-2",
                    Badge { variant: local_variant, "{local_label} (local)" }
                    Badge { variant: cross_ps_variant, "{cross_ps_label} (cross-PS)" }
                }
            }
            p { class: "text-xs font-mono break-all", "receipt_id: {receipt.receipt_id}" }
            div { class: "space-y-1",
                div { class: "flex items-center justify-between text-[11px] text-muted-foreground",
                    span { "Cross-PS acknowledgements" }
                    span { "{done}/{total}" }
                }
                div {
                    class: "h-2 overflow-hidden rounded-full bg-muted",
                    role: "progressbar",
                    aria_valuemin: "0",
                    aria_valuemax: "100",
                    aria_valuenow: "{progress}",
                    aria_label: t("deactivation.fanout_progress_aria"),
                    div {
                        class: "h-full rounded-full bg-primary transition-all",
                        style: "width: {progress}%;"
                    }
                }
            }
            if receipt.cross_ps_peers.is_empty() {
                p { class: "text-xs text-muted-foreground",
                    "No peer-level cross-PS receipt evidence was returned by the principal server."
                }
            } else {
                ul { class: "space-y-1",
                    for peer in receipt.cross_ps_peers.iter() {
                        {
                            let state = peer.state;
                            let variant = state.badge_variant();
                            let label = state.label();
                            let observed = format_optional_iso_datetime(peer.observed_at.as_deref());
                            rsx! {
                                li { class: "flex flex-wrap items-center gap-2 rounded-md border px-2 py-1 text-xs",
                                    span { class: "font-mono break-all", "{peer.service_did}" }
                                    Badge { variant: variant, "{label}" }
                                    span { class: "ml-auto text-muted-foreground", "{observed}" }
                                    if let Some(code) = peer.error_code.as_ref() {
                                        span { class: "basis-full font-mono text-[10px] text-destructive", "error_code={code}" }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fanout_domain_all_is_seven() {
        assert_eq!(FanoutDomain::all().len(), 7);
    }

    #[test]
    fn progress_counts_only_succeeded() {
        let mut snapshot = placeholder_snapshot("did:web:bob.example");
        let (done, total) = snapshot.progress();
        assert_eq!(total, 7);
        // 5 of 7 succeeded in the placeholder.
        assert_eq!(done, 5);
        assert!(snapshot.any_failed());

        for d in snapshot.domains.iter_mut() {
            d.state = FanoutState::Succeeded;
            d.error_code = None;
        }
        assert_eq!(snapshot.progress(), (7, 7));
        assert!(!snapshot.any_failed());
    }

    #[test]
    fn failed_state_renders_red_chip_variant() {
        assert!(matches!(
            FanoutState::Failed.badge_variant(),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            FanoutState::Succeeded.badge_variant(),
            BadgeVariant::Success
        ));
    }
}
