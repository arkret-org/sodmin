//! Round R2/R3 — Deactivation review page (T07).
//!
//! Lands the operator on a per-subject view of the
//! `cx.identity.deactivate` (or `cx.realm.destroy`) fanout result. The
//! seven-domain fanout panel + an optional erasure-receipt block are
//! rendered by `components::deactivation_fanout_panel`; this page is
//! the route shell that fetches a snapshot and wires the retry handler.

use dioxus::prelude::*;

use crate::components::deactivation_fanout_panel::{
    DeactivationFanoutPanel, FanoutDomain, placeholder_snapshot,
};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::toast::{ToastVariant, show_toast};

#[component]
pub fn DeactivationReviewPage() -> Element {
    let mut snapshot = use_signal(|| placeholder_snapshot("did:web:bob.example"));

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Deactivation review".to_string(),
                description: "Per-subject 7-domain fanout result. Failed domains can be retried independently. Round R2/R3 T07.".to_string(),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        // TODO(round23-T07): re-fetch
                        // /api/admin/v1/identity/deactivations/{id}/describe
                        // for the most recent fanout state.
                        snapshot.set(placeholder_snapshot("did:web:bob.example"));
                        show_toast("Refreshed (placeholder).", ToastVariant::Default);
                    },
                    "Refresh"
                }
            }
            {
                let snap = snapshot.read().clone();
                rsx! {
                    DeactivationFanoutPanel {
                        snapshot: snap,
                        on_retry: move |domain: FanoutDomain| {
                            // TODO(round23-T07): POST
                            // /api/admin/v1/identity/deactivations/{id}/retry
                            // with body `{domain: "<slug>"}`.
                            show_toast(
                                &format!("Retry queued for `{}` (placeholder).", domain.slug()),
                                ToastVariant::Default,
                            );
                        }
                    }
                }
            }
        }
    }
}
