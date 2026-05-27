//! R3 (UI-4) — Recovery policy admin surface (stub).
//!
//! Lists `cx.coauth.recovery.*` policies, lets the operator inspect /
//! rotate one, and renders the recovery receipt history including
//! `proof_summary[]`.
//!
//! The data API ships in R3.1 (`/api/admin/v1/recovery/policies`,
//! `/api/admin/v1/recovery/policies/{id}/rotate`,
//! `/api/admin/v1/recovery/receipts`). Until then this page renders the
//! wire surface as a "coming soon" stub so the route is exercised by
//! the SPA and operators see where the workflow will live.

use dioxus::prelude::*;

use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::utils::i18n::t;

/// Stub recovery policy row — replaced in R3.1 by the soland response
/// shape (`RecoveryPolicy { id, kind, status, … }`).
struct StubPolicy {
    id: &'static str,
    kind: &'static str,
    status: &'static str,
}

const STUB_POLICIES: &[StubPolicy] = &[
    StubPolicy {
        id: "cx:recovery:passphrase",
        kind: "passphrase",
        status: "active",
    },
    StubPolicy {
        id: "cx:recovery:hardware-key",
        kind: "hardware_key",
        status: "active",
    },
];

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
                }
                CardContent {
                    p { class: "text-xs text-muted-foreground mb-2",
                        {t("recovery.placeholder")}
                    }
                    Table {
                        TableHeader {
                            TableRow {
                                TableHead { {t("recovery.policy_id")} }
                                TableHead { {t("recovery.policy_type")} }
                                TableHead { {t("recovery.policy_status")} }
                                TableHead { class: "text-right".to_string(), {t("recovery.policy_actions")} }
                            }
                        }
                        TableBody {
                            for policy in STUB_POLICIES.iter() {
                                TableRow {
                                    TableCell { class: "font-mono text-xs".to_string(), "{policy.id}" }
                                    TableCell { "{policy.kind}" }
                                    TableCell {
                                        Badge { variant: BadgeVariant::Success, "{policy.status}" }
                                    }
                                    TableCell { class: "text-right".to_string(),
                                        div { class: "flex justify-end gap-1",
                                            Button {
                                                variant: ButtonVariant::Outline,
                                                size: ButtonSize::Sm,
                                                onclick: |_| show_toast("Inspect: wired in R3.1", ToastVariant::Default),
                                                {t("recovery.inspect")}
                                            }
                                            Button {
                                                variant: ButtonVariant::Outline,
                                                size: ButtonSize::Sm,
                                                onclick: |_| show_toast("Rotate: wired in R3.1", ToastVariant::Default),
                                                {t("recovery.rotate")}
                                            }
                                        }
                                    }
                                }
                            }
                        }
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
                    // `/api/admin/v1/recovery/receipts`.
                }
            }
        }
    }
}
