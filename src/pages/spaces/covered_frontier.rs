//! E2EE covered_frontier lag admin page (Stream H', H'7).
//!
//! Renders the snapshot returned by
//! `GET /api/admin/v1/spaces/{id}/mls/covered-frontier` and shows how
//! many governance Moves the MLS group has yet to acknowledge. Above the
//! configurable threshold the lag count is painted in destructive red
//! with a warning banner so the admin sees the urgency, AND a
//! "Manually advance covered_frontier" override button is surfaced so
//! the operator can fold the current governance frontier into the MLS
//! cover or-set when members are stuck offline. The override POSTs to
//! `/api/admin/v1/spaces/{id}/mls/covered-frontier/advance` and follows
//! the same 404-tolerant pattern as the other Stream H' admin actions.

use dioxus::prelude::*;

use crate::api::covered_frontier_admin;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::covered_frontier::DEFAULT_LAG_WARN_THRESHOLD;
use crate::utils::error::format_optional_endpoint_error;

#[component]
pub fn CoveredFrontierPage(space_id: String) -> Element {
    if is_placeholder_resource_id(&space_id) {
        return selection_required_state("Space");
    }

    let space_id_for_fetch = space_id.clone();
    let mut data = use_resource(move || {
        let id = space_id_for_fetch.clone();
        async move { covered_frontier_admin::get_covered_frontier(&id).await }
    });
    let mut advancing = use_signal(|| false);
    let header_space_id = space_id.clone();
    let space_id_for_action = space_id.clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: format!("covered_frontier · {}", header_space_id),
                description: "Governance frontier vs MLS group epoch — Move acknowledgement lag.".to_string(),
            }

            match &*data.read() {
                Some(Ok(snap)) => {
                    let lag = snap.lag_count();
                    let above_threshold = snap.lag_above(DEFAULT_LAG_WARN_THRESHOLD);
                    let (lag_variant, lag_label) = lag_badge(lag, above_threshold);
                    let mls_epoch = snap.mls_epoch;
                    let governance_count = snap.governance_frontier.len();
                    let covered_count = snap.covered_frontier.len();
                    let governance_empty = snap.governance_frontier.is_empty()
                        && snap.covered_frontier.is_empty()
                        && snap.latest_anchor_id.is_none();
                    let frontier_text = snap.governance_frontier.join(", ");
                    let covered_text = snap.covered_frontier.join(", ");
                    let last_anchor = snap
                        .latest_anchor_id
                        .clone()
                        .unwrap_or_else(|| "-".to_string());
                    let last_covered_at = snap
                        .last_covered_at
                        .clone()
                        .unwrap_or_else(|| "-".to_string());
                    if governance_empty {
                        rsx! {
                            EmptyState {
                                icon: "shield".to_string(),
                                title: "No covered_frontier data yet".to_string(),
                                description: "soland has not yet seen any governance Moves for this Space — covered_frontier is empty by construction.".to_string(),
                            }
                        }
                    } else {
                        let space_id_for_button = space_id_for_action.clone();
                        let advancing_now = *advancing.read();
                        rsx! {
                            if above_threshold {
                                div { class: "rounded-md bg-destructive/10 p-3 text-sm text-destructive flex items-center justify-between gap-3",
                                    span {
                                        {format!(
                                            "Lag of {lag} Moves is above warn threshold {DEFAULT_LAG_WARN_THRESHOLD}; investigate MLS group health (member offline, KeyPackage stale)."
                                        )}
                                    }
                                    Button {
                                        variant: ButtonVariant::Destructive,
                                        disabled: advancing_now,
                                        onclick: move |_| {
                                            let id = space_id_for_button.clone();
                                            advancing.set(true);
                                            spawn(async move {
                                                let res = covered_frontier_admin::advance(&id).await;
                                                match res {
                                                    Ok(r) => show_toast(
                                                        &format!("Advanced covered_frontier; new lag = {}", r.lag_count),
                                                        ToastVariant::Success,
                                                    ),
                                                    Err(e) => {
                                                        let msg = format_optional_endpoint_error(
                                                            "covered_frontier advance",
                                                            &e,
                                                        );
                                                        show_toast(&msg, ToastVariant::Error);
                                                    }
                                                }
                                                advancing.set(false);
                                                data.restart();
                                            });
                                        },
                                        if advancing_now { "Advancing…" } else { "Manually advance covered_frontier" }
                                    }
                                }
                            }
                            Card {
                                CardHeader { CardTitle { "Lag summary" } }
                                CardContent {
                                    div { class: "space-y-2 text-sm",
                                        div { class: "flex items-center gap-2",
                                            span { class: "text-muted-foreground", "Lag:" }
                                            Badge { variant: lag_variant, "{lag_label}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground mr-2", "MLS epoch:" }
                                            span { class: "font-mono text-xs", "{mls_epoch}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground mr-2", "Governance frontier size:" }
                                            span { class: "font-mono text-xs", "{governance_count}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground mr-2", "Covered frontier size:" }
                                            span { class: "font-mono text-xs", "{covered_count}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground mr-2", "Latest anchor:" }
                                            span { class: "font-mono text-xs", "{last_anchor}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground mr-2", "Last covered_frontier update:" }
                                            span { class: "font-mono text-xs", "{last_covered_at}" }
                                        }
                                    }
                                }
                            }

                            Card {
                                CardHeader { CardTitle { "Governance frontier" } }
                                CardContent {
                                    p { class: "font-mono text-xs break-all", "{frontier_text}" }
                                }
                            }

                            Card {
                                CardHeader { CardTitle { "Covered frontier" } }
                                CardContent {
                                    p { class: "font-mono text-xs break-all", "{covered_text}" }
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

/// Badge for the lag count: destructive when above threshold, success
/// when zero, secondary in between. Pure helper so we can unit-test the
/// exact variant assignment without rendering.
pub(crate) fn lag_badge(lag: u64, above_threshold: bool) -> (BadgeVariant, String) {
    let variant = if above_threshold {
        BadgeVariant::Destructive
    } else if lag == 0 {
        BadgeVariant::Success
    } else {
        BadgeVariant::Secondary
    };
    (variant, format!("{lag} Move(s)"))
}

#[cfg(test)]
mod tests {
    use super::lag_badge;
    use crate::components::ui::badge::BadgeVariant;

    #[test]
    fn lag_badge_zero_is_success() {
        let (v, l) = lag_badge(0, false);
        assert!(matches!(v, BadgeVariant::Success));
        assert_eq!(l, "0 Move(s)");
    }

    #[test]
    fn lag_badge_below_threshold_is_secondary() {
        let (v, _) = lag_badge(2, false);
        assert!(matches!(v, BadgeVariant::Secondary));
    }

    #[test]
    fn lag_badge_above_threshold_is_destructive() {
        let (v, l) = lag_badge(12, true);
        assert!(matches!(v, BadgeVariant::Destructive));
        assert_eq!(l, "12 Move(s)");
    }
}
