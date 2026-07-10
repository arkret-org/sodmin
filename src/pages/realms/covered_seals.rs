//! E2EE covered_seals lag admin page (Stream H', H'7).
//!
//! Renders the snapshot returned by
//! `GET /_soland/admin/realms/{id}/mls/covered-seals` and shows how
//! many governance Seals the MLS group has yet to acknowledge. Above the
//! configurable threshold the lag count is painted in destructive red
//! with a warning banner so the admin sees the urgency, AND a
//! "Manually advance covered_seals" override button is surfaced so
//! the operator can fold the current governance Seal set into the MLS
//! cover or-set when members are stuck offline. The override POSTs to
//! `/_soland/admin/realms/{id}/mls/covered-seals/advance` and follows
//! the same 404-tolerant pattern as the other Stream H' admin actions.

use dioxus::prelude::*;

use crate::api::covered_seals;
use crate::components::dangerous_action_dialog::DangerousActionDialog;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::covered_seals::DEFAULT_LAG_WARN_THRESHOLD;
use crate::utils::i18n::t;
use crate::utils::net::error::format_optional_endpoint_error;

#[component]
pub fn CoveredSealsPage(realm_id: String) -> Element {
    if is_placeholder_resource_id(&realm_id) {
        return selection_required_state("Realm");
    }

    let realm_id_for_fetch = realm_id.clone();
    let mut data = use_resource(move || {
        let id = realm_id_for_fetch.clone();
        async move { covered_seals::get_covered_seals(&id).await }
    });
    let mut advancing = use_signal(|| false);
    let mut show_advance_confirm = use_signal(|| false);
    let header_realm_id = realm_id.clone();
    let realm_id_for_action = realm_id.clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("realm_covered_seals.title").replace("{realm_id}", &header_realm_id),
                description: t("realm_covered_seals.description"),
            }

            match &*data.read() {
                Some(Ok(snap)) => {
                    let lag = snap.lag_count();
                    let above_threshold = snap.lag_above(DEFAULT_LAG_WARN_THRESHOLD);
                    let lag_variant = lag_badge_variant(lag, above_threshold);
                    let lag_label =
                        t("realm_covered_seals.lag_badge").replace("{lag}", &lag.to_string());
                    let mls_epoch = snap.mls_epoch;
                    let governance_count = snap.governance_seals.len();
                    let covered_count = snap.covered_seals.len();
                    let governance_empty = snap.governance_seals.is_empty()
                        && snap.covered_seals.is_empty()
                        && snap.latest_seal_id.is_none();
                    let governance_text = snap.governance_seals.join(", ");
                    let covered_text = snap.covered_seals.join(", ");
                    let latest_seal = snap
                        .latest_seal_id
                        .clone()
                        .unwrap_or_else(|| "-".to_string());
                    let last_covered_at = snap
                        .last_covered_at
                        .clone()
                        .unwrap_or_else(|| "-".to_string());
                    if governance_empty {
                        rsx! {
                            EmptyState {
                                icon_name: "shield".to_string(),
                                title: t("realm_covered_seals.empty_title"),
                                description: t("realm_covered_seals.empty_description"),
                            }
                        }
                    } else {
                        let advancing_now = *advancing.read();
                        rsx! {
                            if above_threshold {
                                div { class: "rounded-md bg-destructive/10 p-3 text-sm text-destructive flex items-center justify-between gap-3",
                                    span {
                                        {t("realm_covered_seals.warn_banner")
                                            .replace("{lag}", &lag.to_string())
                                            .replace("{threshold}", &DEFAULT_LAG_WARN_THRESHOLD.to_string())}
                                    }
                                    Button {
                                        variant: ButtonVariant::Destructive,
                                        disabled: advancing_now,
                                        onclick: move |_| show_advance_confirm.set(true),
                                        if advancing_now { {t("realm_covered_seals.advancing")} } else { {t("realm_covered_seals.advance_button")} }
                                    }
                                }
                            }
                            Card {
                                CardHeader { CardTitle { {t("realm_covered_seals.lag_summary_title")} } }
                                CardContent {
                                    div { class: "space-y-2 text-sm",
                                        div { class: "flex items-center gap-2",
                                            span { class: "text-muted-foreground", {t("realm_covered_seals.label_lag")} }
                                            Badge { variant: lag_variant, "{lag_label}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground mr-2", {t("realm_covered_seals.label_mls_epoch")} }
                                            span { class: "font-mono text-xs", "{mls_epoch}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground mr-2", {t("realm_covered_seals.label_governance_size")} }
                                            span { class: "font-mono text-xs", "{governance_count}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground mr-2", {t("realm_covered_seals.label_covered_size")} }
                                            span { class: "font-mono text-xs", "{covered_count}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground mr-2", {t("realm_covered_seals.label_latest_seal")} }
                                            span { class: "font-mono text-xs", "{latest_seal}" }
                                        }
                                        div {
                                            span { class: "text-muted-foreground mr-2", {t("realm_covered_seals.label_last_update")} }
                                            span { class: "font-mono text-xs", "{last_covered_at}" }
                                        }
                                    }
                                }
                            }

                            Card {
                                CardHeader { CardTitle { {t("realm_covered_seals.governance_title")} } }
                                CardContent {
                                    p { class: "font-mono text-xs break-all", "{governance_text}" }
                                }
                            }

                            Card {
                                CardHeader { CardTitle { {t("realm_covered_seals.covered_title")} } }
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

            DangerousActionDialog {
                open: *show_advance_confirm.read(),
                confirmation_phrase: "ADVANCE".to_string(),
                title: t("realm_covered_seals.confirm_title"),
                description: t("realm_covered_seals.confirm_description")
                    .replace("{realm_id}", &realm_id_for_action),
                confirm_text: t("realm_covered_seals.confirm_button"),
                cancel_text: t("common.cancel"),
                on_cancel: move |_| show_advance_confirm.set(false),
                on_confirm: move |_| {
                    if *advancing.read() {
                        return;
                    }
                    show_advance_confirm.set(false);
                    let id = realm_id_for_action.clone();
                    advancing.set(true);
                    spawn(async move {
                        let res = covered_seals::advance(&id).await;
                        match res {
                            Ok(r) => show_toast(
                                &t("realm_covered_seals.toast_advanced")
                                    .replace("{lag}", &r.lag_count.to_string()),
                                ToastVariant::Success,
                            ),
                            Err(e) => {
                                let msg = format_optional_endpoint_error(
                                    "covered_seals advance",
                                    &e,
                                );
                                show_toast(&msg, ToastVariant::Error);
                            }
                        }
                        advancing.set(false);
                        data.restart();
                    });
                },
            }
        }
    }
}

/// Badge variant for the lag count: destructive when above threshold,
/// success when zero, secondary in between. Pure helper so we can
/// unit-test the exact variant assignment without rendering. The
/// user-facing badge label is built at the render site via i18n
/// (`realm_covered_seals.lag_badge`).
pub(crate) fn lag_badge_variant(lag: u64, above_threshold: bool) -> BadgeVariant {
    if above_threshold {
        BadgeVariant::Destructive
    } else if lag == 0 {
        BadgeVariant::Success
    } else {
        BadgeVariant::Secondary
    }
}

#[cfg(test)]
mod tests {
    use super::lag_badge_variant;
    use crate::components::ui::badge::BadgeVariant;

    #[test]
    fn lag_badge_zero_is_success() {
        let v = lag_badge_variant(0, false);
        assert!(matches!(v, BadgeVariant::Success));
    }

    #[test]
    fn lag_badge_below_threshold_is_secondary() {
        let v = lag_badge_variant(2, false);
        assert!(matches!(v, BadgeVariant::Secondary));
    }

    #[test]
    fn lag_badge_above_threshold_is_destructive() {
        let v = lag_badge_variant(12, true);
        assert!(matches!(v, BadgeVariant::Destructive));
    }
}
