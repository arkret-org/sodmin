//! Round R2/R3 — 7-domain deactivation fanout panel (T07).
//!
//! When an admin runs `cx.identity.deactivate` (or `cx.realm.destroy`),
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
//! For `cx.realm.destroy` the same panel is reused; in addition the
//! caller wires an "erasure receipt" cross-PS bar. Cross-PS visibility
//! is `TODO(round23-T07)` — the panel only shows the local PS result.

use dioxus::prelude::*;

use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::*;

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
    /// Last error code (if state == Failed) — typically one of the new
    /// round 2+3 errors (e.g. `cx.error.fanout_partial`).
    pub error_code: Option<String>,
    pub attempt_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanoutState {
    Pending,
    InProgress,
    Succeeded,
    Failed,
}

impl FanoutState {
    pub fn badge_variant(&self) -> BadgeVariant {
        match self {
            FanoutState::Pending => BadgeVariant::Secondary,
            FanoutState::InProgress => BadgeVariant::Default,
            FanoutState::Succeeded => BadgeVariant::Success,
            FanoutState::Failed => BadgeVariant::Destructive,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            FanoutState::Pending => "pending",
            FanoutState::InProgress => "in_progress",
            FanoutState::Succeeded => "ok",
            FanoutState::Failed => "failed",
        }
    }
}

/// Top-level snapshot of all seven domains for one
/// `cx.identity.deactivate` (or `cx.realm.destroy`) invocation.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FanoutSnapshot {
    pub subject_id: String,
    pub domains: Vec<FanoutDomainResult>,
    /// Only present on `cx.realm.destroy`. For `cx.identity.deactivate`
    /// this is `None`.
    pub erasure_receipt: Option<ErasureReceiptStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErasureReceiptStatus {
    pub receipt_id: String,
    pub local_state: FanoutState,
    /// Cross-PS aggregate state. Always `None` in this round — the
    /// principal server does not yet expose this surface; the panel
    /// renders a TODO placeholder.
    pub cross_ps_state: Option<FanoutState>,
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
                FanoutDomain::Push => Some("cx.error.fanout_partial".to_string()),
                FanoutDomain::ToDevice => Some("cx.error.fanout_partial".to_string()),
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
    /// this to `POST /api/admin/v1/identity/deactivations/{id}/retry`
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

                // Erasure receipt — only present for `cx.realm.destroy`.
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
    rsx! {
        div { class: "rounded-md border p-3 space-y-2",
            div { class: "flex items-center justify-between gap-2",
                p { class: "text-sm font-semibold", "Erasure receipt" }
                Badge { variant: local_variant, "{local_label} (local)" }
            }
            p { class: "text-xs font-mono break-all", "receipt_id: {receipt.receipt_id}" }
            // TODO(round23-T07): cross-PS visualization — fan out to
            // each federated principal server and render their erasure
            // receipt aggregation here. Today we only show the local PS
            // result.
            p { class: "text-xs text-muted-foreground",
                "Cross-PS progress is not yet wired into sodmin; only the local principal server's erasure receipt is shown. (TODO round23-T07)"
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
