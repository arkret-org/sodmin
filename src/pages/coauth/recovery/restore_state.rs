//! Restore state-machine view (Round 25, C3).
//!
//! Shows the linear restore-state progression for a single ticket:
//! pending → approved → executor_running → complete. Each node renders
//! as a step indicator with `entered_at` / `completed_at` plus an
//! optional note. Reads from the per-ticket detail endpoint already
//! exposed by C2 — `RecoveryTicketDetail.restore_state` is the
//! machine projection.

use dioxus::prelude::*;

use crate::api::recovery_admin;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::router::Route;
use crate::utils::i18n::t;
use coauth_admin_types::recovery_admin::{RecoveryTicketStatus, RestoreStateNode};

/// Ordered canonical projection of the restore-state machine. The page
/// keeps this list stable so the indicator doesn't reorder rows
/// depending on which states soland has emitted timestamps for.
pub(crate) const CANONICAL_STATES: &[RecoveryTicketStatus] = &[
    RecoveryTicketStatus::Pending,
    RecoveryTicketStatus::Approved,
    RecoveryTicketStatus::ExecutorRunning,
    RecoveryTicketStatus::Complete,
];

/// Status-tone for one step in the restore-state visualizer. Consumed
/// by the renderer to pick the badge variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StateTone {
    Done,
    Active,
    Future,
}

impl StateTone {
    pub(crate) fn variant(&self) -> BadgeVariant {
        match self {
            StateTone::Done => BadgeVariant::Success,
            StateTone::Active => BadgeVariant::Default,
            StateTone::Future => BadgeVariant::Secondary,
        }
    }
}

/// Pure helper — pick a tone for `state` given the current ticket
/// status. The list is single-direction (no rollback) so any state
/// before `current` is `Done`, the matching state is `Active`,
/// everything after is `Future`. Terminal failure / rejection /
/// cancellation marks the matching cell as `Active` and the rest as
/// `Future` (the indicator stops moving).
pub(crate) fn tone_for(state: &RecoveryTicketStatus, current: &RecoveryTicketStatus) -> StateTone {
    let pos = canonical_position(state);
    let cur_pos = canonical_position(current);
    match (pos, cur_pos) {
        (Some(p), Some(c)) if p < c => StateTone::Done,
        (Some(p), Some(c)) if p == c => StateTone::Active,
        (Some(_), Some(_)) => StateTone::Future,
        // Off-canonical state (Cancelled / Rejected / Failed): mark
        // every canonical step before current-status as Done; the
        // active state itself is whatever the ticket landed on (which
        // is off-canonical so it doesn't render as a step). Future
        // steps stay Future.
        (Some(_), None) => StateTone::Future,
        (None, _) => StateTone::Future,
    }
}

fn canonical_position(state: &RecoveryTicketStatus) -> Option<usize> {
    CANONICAL_STATES.iter().position(|s| s == state)
}

#[component]
pub fn RestoreStatePage(ticket_id: String) -> Element {
    let id_for_resource = ticket_id.clone();
    let mut data = use_resource(move || {
        let id = id_for_resource.clone();
        async move { recovery_admin::get_ticket(&id).await }
    });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("restore_state.title"),
                description: t("restore_state.subtitle"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }

            div { class: "text-sm",
                Link {
                    to: Route::RecoveryTicketDetail { ticket_id: ticket_id.clone() },
                    class: "text-primary hover:underline".to_string(),
                    {t("restore_state.back_to_detail")}
                }
            }

            match &*data.read() {
                Some(Ok(detail)) => {
                    let current = detail.ticket.status_typed();
                    let restore_state = detail.restore_state.clone();
                    rsx! {
                        ol { class: "space-y-2",
                            for state in CANONICAL_STATES.iter() {
                                {
                                    let tone = tone_for(state, &current);
                                    let label = state.label().to_string();
                                    let node = matching_node(&restore_state, state);
                                    let entered = node
                                        .as_ref()
                                        .and_then(|n| n.entered_at.clone())
                                        .unwrap_or_default();
                                    let completed = node
                                        .as_ref()
                                        .and_then(|n| n.completed_at.clone())
                                        .unwrap_or_default();
                                    let note = node
                                        .as_ref()
                                        .and_then(|n| n.note.clone())
                                        .unwrap_or_default();
                                    rsx! {
                                        li { class: "rounded-md border p-3 flex flex-col gap-1",
                                            div { class: "flex items-center gap-2",
                                                Badge { variant: tone.variant(), "{label}" }
                                                if !entered.is_empty() {
                                                    span { class: "text-xs text-muted-foreground font-mono",
                                                        {format!("{}: {entered}", t("restore_state.entered_at"))}
                                                    }
                                                }
                                                if !completed.is_empty() {
                                                    span { class: "text-xs text-muted-foreground font-mono",
                                                        {format!("{}: {completed}", t("restore_state.completed_at"))}
                                                    }
                                                }
                                            }
                                            if !note.is_empty() {
                                                p { class: "text-xs text-muted-foreground", "{note}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Some(Err(e)) => rsx! {
                    ErrorBanner {
                        message: e.message.clone(),
                        on_retry: move |_| data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}

/// Look up the per-state node soland emitted (if any) for a given
/// canonical state.
pub(crate) fn matching_node<'a>(
    nodes: &'a [RestoreStateNode],
    state: &RecoveryTicketStatus,
) -> Option<&'a RestoreStateNode> {
    nodes.iter().find(|n| &n.state_typed() == state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tone_orders_done_active_future() {
        // current = Approved → Pending=Done, Approved=Active, ExecutorRunning=Future
        assert_eq!(
            tone_for(
                &RecoveryTicketStatus::Pending,
                &RecoveryTicketStatus::Approved
            ),
            StateTone::Done
        );
        assert_eq!(
            tone_for(
                &RecoveryTicketStatus::Approved,
                &RecoveryTicketStatus::Approved
            ),
            StateTone::Active
        );
        assert_eq!(
            tone_for(
                &RecoveryTicketStatus::ExecutorRunning,
                &RecoveryTicketStatus::Approved,
            ),
            StateTone::Future
        );
        assert_eq!(
            tone_for(
                &RecoveryTicketStatus::Complete,
                &RecoveryTicketStatus::Approved
            ),
            StateTone::Future
        );
    }

    #[test]
    fn off_canonical_states_freeze_progression() {
        // current = Cancelled → all canonical steps render as Future
        // (the indicator stops moving once we leave the happy path).
        for s in CANONICAL_STATES.iter() {
            assert_eq!(
                tone_for(s, &RecoveryTicketStatus::Cancelled),
                StateTone::Future
            );
        }
    }

    #[test]
    fn matching_node_finds_canonical_state() {
        let nodes = vec![
            RestoreStateNode {
                state: "approved".into(),
                entered_at: Some("t1".into()),
                ..Default::default()
            },
            RestoreStateNode {
                state: "pending".into(),
                entered_at: Some("t0".into()),
                ..Default::default()
            },
        ];
        let n = matching_node(&nodes, &RecoveryTicketStatus::Approved).unwrap();
        assert_eq!(n.entered_at.as_deref(), Some("t1"));
        let n = matching_node(&nodes, &RecoveryTicketStatus::Pending).unwrap();
        assert_eq!(n.entered_at.as_deref(), Some("t0"));
        assert!(matching_node(&nodes, &RecoveryTicketStatus::Complete).is_none());
    }
}
