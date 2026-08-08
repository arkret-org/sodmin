//! Read-only multi-sig aggregation status panel.
//!
//! Lists pending Seals for a Realm's notary cell that's configured as
//! `threshold(k of n)` or `mixed`. Each row shows the seal_id, the
//! `k of n` threshold, the count of partials collected, and the missing
//! signer DIDs. Signature authoring and submission belongs in a client
//! that holds the corresponding notary key.

use dioxus::prelude::*;

use crate::api::multisig;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::types::multisig::PendingMultisigSeal;
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
                                    }
                                }
                                TableBody {
                                    for entry in pending.iter() {
                                        {
                                            let seal_id = entry.seal_id.clone();
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
