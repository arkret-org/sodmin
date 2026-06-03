//! R3 (UI-4) — Recovery policy admin surface (stub).
//!
//! Lists `ck.coauth.recovery.*` policies, lets the operator inspect /
//! rotate one, and renders the recovery receipt history including
//! `proof_summary[]`.
//!
//! The data API ships in R3.1 (`/_soland/admin/recovery/policies`,
//! `/_soland/admin/recovery/policies/{id}/rotate`,
//! `/_soland/admin/recovery/receipts`). Until then this page renders the
//! wire surface as a "coming soon" stub so the route is exercised by
//! the SPA and operators see where the workflow will live.

use dioxus::prelude::*;

use crate::components::ui::card::*;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::page_header::PageHeader;
use crate::utils::i18n::t;

#[component]
pub fn RecoveryPolicyList() -> Element {
    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("recovery.title"),
                description: t("recovery.subtitle"),
            }

            // TODO(R3.1): replace with real `recovery::list_policies()` call
            // once the soland endpoint lands.
            Card {
                CardHeader {
                    CardTitle { {t("recovery.list_title")} }
                    CardDescription {
                        "No synthetic recovery policies are rendered. Wire the coauth recovery policy endpoint before enabling inspect or rotate actions."
                    }
                }
                CardContent {
                    EmptyState {
                        icon: "key".to_string(),
                        title: "Recovery policy endpoint unavailable".to_string(),
                        description: t("recovery.placeholder"),
                    }
                }
            }

            Card {
                CardHeader {
                    CardTitle { {t("recovery.receipts_title")} }
                    CardDescription { {t("recovery.receipts_subtitle")} }
                }
                CardContent {
                    p { class: "text-sm text-muted-foreground py-6 text-center",
                        {t("recovery.placeholder")}
                    }
                    // TODO(R3.1): render the recovery_receipt history table
                    // with one row per receipt and an expandable
                    // proof_summary[] subtable (proof_type / verifier /
                    // verified_at). Receipts originate from
                    // `/_soland/admin/recovery/receipts`.
                }
            }
        }
    }
}
