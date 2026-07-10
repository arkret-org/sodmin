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

#[component]
pub fn DeactivationReviewPage() -> Element {
    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Deactivation review".to_string(),
                description: "Per-subject 7-domain fanout result. Failed domains can be retried independently. Round R2/R3 T07.".to_string(),
                Button {
                    variant: ButtonVariant::Outline,
                    disabled: true,
                    "Refresh"
                }
            }

            div {
                class: "rounded-md border border-amber-600 bg-amber-600/10 px-3 py-2 text-sm",
                role: "alert",
                p { class: "font-semibold text-amber-700 dark:text-amber-200",
                    "Deactivation fanout describe endpoint is not available yet."
                }
                p { class: "text-xs text-amber-700/90 dark:text-amber-200/90",
                    "This page no longer renders synthetic fanout data. Wire `/_soland/admin/identity/deactivations/<id>/describe` before enabling refresh or retry."
                }
            }

            EmptyState {
                icon_name: "alert-triangle".to_string(),
                title: "No deactivation selected".to_string(),
                description: "Open this workflow from an actual account or Realm deactivation record once the backend describe endpoint is published.".to_string(),
            }
        }
    }
}
