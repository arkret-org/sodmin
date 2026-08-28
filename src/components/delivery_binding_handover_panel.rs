//! Delivery-binding handover panel.
//!
//! Renders the handover error-code triple
//! (`delivery_binding_stale` / `delivery_binding_handed_over` /
//! `historical_only`) as discrete rows so the operator can:
//!
//! - See exactly which envelope the recipient service rejected, and
//! - Read the `new_recipient_id` + `handover_frontier` the recipient advertises in the 409 body
//!   (`stale` / `handed_over`).
//! - Distinguish a fresh failure from a `historical_only` cached replay — the latter is diagnostic
//!   only and MUST NOT be presented as a "fresh action" indicator.

use arkret_wire::ErrorCode;
use dioxus::prelude::*;

use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::card::*;
use crate::types::DeliveryBindingHandoverRow;
use crate::utils::i18n::t;

#[derive(Props, Clone, PartialEq)]
pub struct DeliveryBindingHandoverPanelProps {
    pub rows: Vec<DeliveryBindingHandoverRow>,
}

#[component]
pub fn DeliveryBindingHandoverPanel(props: DeliveryBindingHandoverPanelProps) -> Element {
    let rows = props.rows.clone();
    rsx! {
        Card {
            CardHeader {
                CardTitle { class: "text-lg".to_string(), {t("delivery_binding.handover.title")} }
                CardDescription { {t("delivery_binding.handover.subtitle")} }
            }
            CardContent { class: "space-y-3".to_string(),
                if rows.is_empty() {
                    p { class: "text-sm text-muted-foreground py-4 text-center",
                        "No delivery-binding handover events observed."
                    }
                } else {
                    ul { class: "space-y-2",
                        for row in rows.iter() {
                            {render_row(row)}
                        }
                    }
                }
            }
        }
    }
}

fn render_row(row: &DeliveryBindingHandoverRow) -> Element {
    let reason = row.classified_reason();
    let (border_class, badge_variant, code_label, explainer_key) = match reason {
        Some(ErrorCode::DeliveryBindingStale) => (
            "border-red-600/40 bg-red-600/10",
            BadgeVariant::Destructive,
            "delivery_binding_stale".to_string(),
            "delivery_binding.handover.stale_explainer",
        ),
        Some(ErrorCode::DeliveryBindingHandedOver) => (
            "border-red-600/40 bg-red-600/10",
            BadgeVariant::Destructive,
            "delivery_binding_handed_over".to_string(),
            "delivery_binding.handover.handed_over_explainer",
        ),
        Some(ErrorCode::HistoricalOnly) => (
            // historical_only is a diagnostic, NOT a fresh-action
            // marker. Use the muted secondary tone so operators don't
            // misread it as a new failure.
            "border-muted bg-muted/30",
            BadgeVariant::Secondary,
            "historical_only".to_string(),
            "delivery_binding.handover.historical_only_explainer",
        ),
        Some(_) | None => (
            "border-muted",
            BadgeVariant::Secondary,
            row.reason_code
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            "delivery_binding.handover.historical_only_explainer",
        ),
    };

    let realm = if row.realm_id.is_empty() {
        "-".to_string()
    } else {
        row.realm_id.clone()
    };
    let actor = if row.actor_id.is_empty() {
        "-".to_string()
    } else {
        row.actor_id.clone()
    };
    let new_recipient = row
        .new_recipient_id
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let prev_recipient = row
        .previous_recipient_id
        .clone()
        .unwrap_or_else(|| "-".to_string());
    let observed_at = row.observed_at.clone().unwrap_or_else(|| "-".to_string());
    // Render the handover_frontier vector as a chip strip;
    // each entry is a `ak:event:*` ref. Long vectors are common, so
    // a compact pill list reads better than an inline string.
    let frontier = row.handover_frontier.clone();

    rsx! {
        li { class: "rounded-md border p-3 text-xs space-y-2 {border_class}",
            div { class: "flex items-center justify-between gap-2",
                div { class: "space-y-0.5",
                    div { class: "font-semibold",
                        {format!("Realm {} · actor {}", realm, actor)}
                    }
                    div { class: "font-mono text-[10px] text-muted-foreground", "observed_at={observed_at}" }
                }
                Badge { variant: badge_variant, class: "font-mono".to_string(), "{code_label}" }
            }
            p { class: "text-[11px] italic text-muted-foreground", {t(explainer_key)} }
            div { class: "grid gap-2 sm:grid-cols-2",
                div {
                    p { class: "text-[10px] uppercase tracking-wider text-muted-foreground",
                        "previous_recipient_id"
                    }
                    p { class: "font-mono break-all", "{prev_recipient}" }
                }
                div {
                    p { class: "text-[10px] uppercase tracking-wider text-muted-foreground",
                        {t("delivery_binding.handover.new_recipient")}
                    }
                    p { class: "font-mono break-all", "{new_recipient}" }
                }
            }
            div { class: "space-y-1",
                p { class: "text-[10px] uppercase tracking-wider text-muted-foreground",
                    {t("delivery_binding.handover.frontier")}
                }
                if frontier.is_empty() {
                    p { class: "text-muted-foreground", "-" }
                } else {
                    div { class: "flex flex-wrap gap-1",
                        for ev in frontier.iter() {
                            Badge { variant: BadgeVariant::Secondary, class: "font-mono text-[10px]".to_string(), "{ev}" }
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

    fn row(reason_code: &str) -> DeliveryBindingHandoverRow {
        DeliveryBindingHandoverRow {
            realm_id: "ak:realm:AWTBjgRH5aE_aJLYZzA-BJ4C0RJCw16DIs9TIhgRAhCz".into(),
            actor_id: "ak:did_core:web:actor.example".into(),
            previous_recipient_id: None,
            new_recipient_id: None,
            handover_frontier: Vec::new(),
            reason_code: Some(reason_code.into()),
            observed_at: None,
        }
    }

    #[test]
    fn classified_reason_maps_the_three_codes() {
        let stale = row("delivery_binding_stale");
        assert_eq!(
            stale.classified_reason(),
            Some(ErrorCode::DeliveryBindingStale)
        );

        let handed_over = row("delivery_binding_handed_over");
        assert_eq!(
            handed_over.classified_reason(),
            Some(ErrorCode::DeliveryBindingHandedOver)
        );

        let historical = row("historical_only");
        assert_eq!(
            historical.classified_reason(),
            Some(ErrorCode::HistoricalOnly)
        );
    }
}
