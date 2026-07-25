//! Multi-sig partial-signature aggregation panel (Stream H', H'9).
//!
//! Lists pending Seals for a Realm's notary cell that's configured as
//! `threshold(k of n)` or `mixed`. Each row shows the seal_id, the
//! `k of n` threshold, the count of partials collected, and the missing
//! signer DIDs. When the current admin DID is in the missing-signers
//! list, soland sets `admin_can_sign=true` on the row and the
//! `Submit my partial signature` button activates.

use dioxus::prelude::*;

use crate::api::multisig;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::multisig::{PartialSubmitStatus, PendingMultisigSeal};
use crate::utils::i18n::t;

#[component]
pub fn MultiSigPage(realm_id: String) -> Element {
    if is_placeholder_resource_id(&realm_id) {
        return selection_required_state("Realm");
    }

    let realm_id_for_fetch = realm_id.clone();
    let mut data = use_resource(move || {
        let id = realm_id_for_fetch.clone();
        async move { multisig::list_pending(&id).await }
    });
    // Per-row in-flight flag keyed by seal_id.
    let mut in_flight = use_signal::<Option<String>>(|| None);
    let header_realm_id = realm_id.clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("realm_multisig.title").replace("{realm_id}", &header_realm_id),
                description: t("realm_multisig.description"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("realm_multisig.refresh")}
                }
            }

            match &*data.read() {
                Some(Ok(pending)) => if pending.is_empty() {
                    rsx! {
                        EmptyState {
                            icon_name: "shield".to_string(),
                            title: t("realm_multisig.empty_title"),
                            description: t("realm_multisig.empty_description"),
                        }
                    }
                } else {
                    rsx! {
                        div { class: "rounded-md border",
                            Table {
                                TableHeader {
                                    TableRow {
                                        TableHead { {t("realm_multisig.col_seal_id")} }
                                        TableHead { {t("realm_multisig.col_threshold")} }
                                        TableHead { {t("realm_multisig.col_collected")} }
                                        TableHead { {t("realm_multisig.col_missing_signers")} }
                                        TableHead { {t("realm_multisig.col_state_root")} }
                                        TableHead { {t("realm_multisig.col_created")} }
                                        TableHead { class: "text-right".to_string(), {t("realm_multisig.col_action")} }
                                    }
                                }
                                TableBody {
                                    for entry in pending.iter() {
                                        {
                                            let seal_id = entry.seal_id.clone();
                                            let seal_id_for_btn = seal_id.clone();
                                            let realm_id_for_btn = realm_id.clone();
                                            let threshold_label = entry.threshold_label();
                                            let collected = entry.collected_partials;
                                            let remaining = entry.remaining();
                                            let collected_label = t("realm_multisig.collected_label")
                                                .replace("{collected}", &collected.to_string())
                                                .replace("{remaining}", &remaining.to_string());
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
                                                .map(|id| id == seal_id)
                                                .unwrap_or(false);
                                            rsx! {
                                                TableRow {
                                                    key: "{seal_id}",
                                                    TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{seal_id}" }
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
                                                                    let pending_seal_id = seal_id_for_btn.clone();
                                                                    let sid = realm_id_for_btn.clone();
                                                                    let Some(signer_did) = prompt_required(&t("realm_multisig.prompt_signer_did")) else {
                                                                        return;
                                                                    };
                                                                    let Some(signature_b64) = prompt_required(&t("realm_multisig.prompt_partial_signature")) else {
                                                                        return;
                                                                    };
                                                                    let Some(kid) = prompt_required(&t("realm_multisig.prompt_key_id")) else {
                                                                        return;
                                                                    };
                                                                    in_flight.set(Some(pending_seal_id.clone()));
                                                                    spawn(async move {
                                                                        let res = multisig::submit_partial(
                                                                            &sid,
                                                                            &pending_seal_id,
                                                                            &signer_did,
                                                                            &signature_b64,
                                                                            &kid,
                                                                        )
                                                                        .await;
                                                                        match res {
                                                                            Ok(r) => {
                                                                                let met_suffix = if r.status == PartialSubmitStatus::Aggregated {
                                                                                    t("realm_multisig.toast_threshold_met_suffix")
                                                                                } else {
                                                                                    String::new()
                                                                                };
                                                                                show_toast(
                                                                                    &t("realm_multisig.toast_partial_recorded")
                                                                                        .replace("{collected}", &r.collected.to_string())
                                                                                        .replace("{threshold}", &r.threshold.to_string())
                                                                                        .replace("{met}", &met_suffix),
                                                                                    ToastVariant::Success,
                                                                                );
                                                                            }
                                                                            Err(e) => show_toast(
                                                                                &e.message,
                                                                                ToastVariant::Error,
                                                                            ),
                                                                        }
                                                                        in_flight.set(None);
                                                                        data.restart();
                                                                    });
                                                                },
                                                                {t("realm_multisig.submit_partial")}
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

fn prompt_required(label: &str) -> Option<String> {
    let value = web_sys::window()?
        .prompt_with_message(label)
        .ok()
        .flatten()?;
    let trimmed = value.trim().to_owned();
    (!trimmed.is_empty()).then_some(trimmed)
}

/// Pick a badge variant for the "collected partials" cell. When the
/// threshold is already met (assembly pending) we show success-green;
/// otherwise the secondary tone signals "still collecting".
pub(crate) fn collected_badge_variant(entry: &PendingMultisigSeal) -> BadgeVariant {
    if entry.is_threshold_met() {
        BadgeVariant::Success
    } else {
        BadgeVariant::Secondary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pending_with(collected: u32, k: u32) -> PendingMultisigSeal {
        PendingMultisigSeal {
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
