//! Round R2/R3 — Deactivation review page (T07).
//!
//! Lands the operator on a per-subject view of the
//! `ak.self.agent.deactivate` (or `ak.realm.destroy`) fanout result. The
//! seven-domain fanout panel + an optional erasure-receipt block are
//! rendered by `components::deactivation_fanout_panel`; this page is
//! the route shell that fetches a snapshot and wires the retry handler.

use dioxus::prelude::*;

use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::page_header::PageHeader;
use crate::utils::i18n::t;

#[component]
pub fn DeactivationReviewPage() -> Element {
    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("deactivation_review.title"),
                description: t("deactivation_review.description"),
                Button {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    {t("common.refresh")}
                }
            }

            div {
                class: "rounded-md border border-amber-600 bg-amber-600/10 px-3 py-2 text-sm",
                role: "alert",
                p { class: "font-semibold text-amber-700 dark:text-amber-200",
                    {t("deactivation_review.endpoint_unavailable")}
                }
                p { class: "text-xs text-amber-700/90 dark:text-amber-200/90",
                    {t("deactivation_review.endpoint_unavailable_detail")}
                }
            }

            EmptyState {
                icon_name: "alert-triangle".to_string(),
                title: t("deactivation_review.empty_title"),
                description: t("deactivation_review.empty_description"),
            }
        }
    }
}
