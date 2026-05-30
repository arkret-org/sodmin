//! Multi-sig partial-signature aggregation panel (Stream H', H'9).
//!
//! Lists pending Anchors for a Space's anchorer cell that's configured as
//! `threshold(k of n)` or `mixed`. Each row shows the anchor_id, the
//! `k of n` threshold, the count of partials collected, and the missing
//! signer DIDs. When the current admin DID is in the missing-signers
//! list, soland sets `admin_can_sign=true` on the row and the
//! `Submit my partial signature` button activates.
//!
//! Follows the 404-tolerant pattern shared with the rest of Stream H'.

use dioxus::prelude::*;

use crate::api::multisig_admin;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::multisig::PendingMultisigAnchor;
use crate::utils::error::format_optional_endpoint_error;

#[component]
pub fn MultiSigPage(space_id: String) -> Element {
    if is_placeholder_resource_id(&space_id) {
        return selection_required_state("Space");
    }

    let space_id_for_fetch = space_id.clone();
    let mut data = use_resource(move || {
        let id = space_id_for_fetch.clone();
        async move { multisig_admin::list_pending(&id).await }
    });
    // Per-row in-flight flag keyed by anchor_id.
    let mut in_flight = use_signal::<Option<String>>(|| None);
    let header_space_id = space_id.clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: format!("Multi-sig pending · {}", header_space_id),
                description: "Anchors awaiting threshold partial signatures from the anchorer cell members.".to_string(),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    "Refresh"
                }
            }

            match &*data.read() {
                Some(Ok(pending)) => if pending.is_empty() {
                    rsx! {
                        EmptyState {
                            icon: "shield".to_string(),
                            title: "No pending multi-sig Anchors".to_string(),
                            description: "All Anchors in this Space have reached threshold and assembled.".to_string(),
                        }
                    }
                } else {
                    rsx! {
                        div { class: "rounded-md border",
                            Table {
                                TableHeader {
                                    TableRow {
                                        TableHead { "Anchor ID" }
                                        TableHead { "Threshold" }
                                        TableHead { "Collected" }
                                        TableHead { "Missing signers" }
                                        TableHead { "State root" }
                                        TableHead { "Created" }
                                        TableHead { class: "text-right".to_string(), "Action" }
                                    }
                                }
                                TableBody {
                                    for entry in pending.iter() {
                                        {
                                            let anchor_id = entry.anchor_id.clone();
                                            let anchor_id_for_btn = anchor_id.clone();
                                            let space_id_for_btn = space_id.clone();
                                            let threshold_label = entry.threshold_label();
                                            let collected = entry.collected_partials;
                                            let remaining = entry.remaining();
                                            let collected_label =
                                                format!("{collected} (need {remaining} more)");
                                            let collected_variant =
                                                collected_badge_variant(entry);
                                            let missing = entry.missing_signers.join(", ");
                                            let missing_display = if missing.is_empty() {
                                                "—".to_string()
                                            } else {
                                                missing
                                            };
                                            let state_root = entry
                                                .state_root
                                                .clone()
                                                .unwrap_or_else(|| "-".to_string());
                                            let created = entry
                                                .created_at
                                                .clone()
                                                .unwrap_or_else(|| "-".to_string());
                                            let admin_can_sign = entry.admin_can_sign;
                                            let row_in_flight = in_flight
                                                .read()
                                                .as_deref()
                                                .map(|id| id == anchor_id)
                                                .unwrap_or(false);
                                            rsx! {
                                                TableRow {
                                                    TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{anchor_id}" }
                                                    TableCell { "{threshold_label}" }
                                                    TableCell {
                                                        Badge { variant: collected_variant, "{collected_label}" }
                                                    }
                                                    TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{missing_display}" }
                                                    TableCell { class: "font-mono text-xs max-w-[200px] truncate".to_string(), "{state_root}" }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{created}" }
                                                    TableCell { class: "text-right".to_string(),
                                                        if admin_can_sign {
                                                            Button {
                                                                variant: ButtonVariant::Default,
                                                                size: ButtonSize::Sm,
                                                                disabled: row_in_flight,
                                                                onclick: move |_| {
                                                                    let aid = anchor_id_for_btn.clone();
                                                                    let sid = space_id_for_btn.clone();
                                                                    in_flight.set(Some(aid.clone()));
                                                                    spawn(async move {
                                                                        let res = multisig_admin::submit_partial(
                                                                            &sid, &aid, None,
                                                                        )
                                                                        .await;
                                                                        match res {
                                                                            Ok(r) => show_toast(
                                                                                &format!(
                                                                                    "Partial recorded: {}/{}{}",
                                                                                    r.collected_partials,
                                                                                    r.threshold_k,
                                                                                    if r.threshold_met { " — threshold met" } else { "" }
                                                                                ),
                                                                                ToastVariant::Success,
                                                                            ),
                                                                            Err(e) => {
                                                                                let msg = format_optional_endpoint_error(
                                                                                    "multisig partial",
                                                                                    &e,
                                                                                );
                                                                                show_toast(&msg, ToastVariant::Error);
                                                                            }
                                                                        }
                                                                        in_flight.set(None);
                                                                        data.restart();
                                                                    });
                                                                },
                                                                "Submit my partial signature"
                                                            }
                                                        } else {
                                                            span { class: "text-xs text-muted-foreground", "—" }
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
                },
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

/// Pick a badge variant for the "collected partials" cell. When the
/// threshold is already met (assembly pending) we show success-green;
/// otherwise the secondary tone signals "still collecting".
pub(crate) fn collected_badge_variant(entry: &PendingMultisigAnchor) -> BadgeVariant {
    if entry.is_threshold_met() {
        BadgeVariant::Success
    } else {
        BadgeVariant::Secondary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending_with(collected: u32, k: u32) -> PendingMultisigAnchor {
        PendingMultisigAnchor {
            threshold_k: k,
            threshold_n: k + 1,
            collected_partials: collected,
            ..Default::default()
        }
    }

    #[test]
    fn collected_variant_is_success_when_threshold_met() {
        let entry = pending_with(2, 2);
        assert!(matches!(
            collected_badge_variant(&entry),
            BadgeVariant::Success
        ));

        let over = pending_with(3, 2);
        assert!(matches!(
            collected_badge_variant(&over),
            BadgeVariant::Success
        ));
    }

    #[test]
    fn collected_variant_is_secondary_when_still_collecting() {
        let entry = pending_with(1, 3);
        assert!(matches!(
            collected_badge_variant(&entry),
            BadgeVariant::Secondary
        ));

        let zero = pending_with(0, 2);
        assert!(matches!(
            collected_badge_variant(&zero),
            BadgeVariant::Secondary
        ));
    }
}
