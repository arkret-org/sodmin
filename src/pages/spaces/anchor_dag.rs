//! Anchor DAG / compaction admin page (Stream H', H'4).
//!
//! Visualises the latest Anchor leaves, the current frontier, the latest
//! `state_root` and exposes a "trigger compaction" button that POSTs (today,
//! stub-POSTs) to soland's `cx.admin.anchors.sign` endpoint
//! (`POST /api/v1/admin/anchors/sign`).

use dioxus::prelude::*;

use crate::api::anchor_admin;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};

#[component]
pub fn AnchorDagPage(space_id: String) -> Element {
    let space_id_for_fetch = space_id.clone();
    let mut data = use_resource(move || {
        let id = space_id_for_fetch.clone();
        async move { anchor_admin::get_anchor_dag(&id).await }
    });

    let mut compacting = use_signal(|| false);
    let space_id_for_compact = space_id.clone();
    let header_space_id = space_id.clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: format!("Anchor DAG · {}", header_space_id),
                description: "Visualize Anchor leaves, frontier and state_root for this Space.".to_string(),
                Button {
                    variant: ButtonVariant::Default,
                    disabled: *compacting.read(),
                    onclick: move |_| {
                        compacting.set(true);
                        let id = space_id_for_compact.clone();
                        spawn(async move {
                            match anchor_admin::trigger_compaction(&id).await {
                                Ok(r) => show_toast(
                                    &format!("Compaction Anchor signed: {}", r.anchor_id),
                                    ToastVariant::Success,
                                ),
                                Err(e) => show_toast(
                                    &format!("Failed: {}", e.message),
                                    ToastVariant::Error,
                                ),
                            }
                            compacting.set(false);
                        });
                    },
                    "Trigger compaction"
                }
            }

            match &*data.read() {
                Some(Ok(snapshot)) => {
                    let frontier = snapshot.frontier.join(", ");
                    let state_root = snapshot
                        .state_root
                        .clone()
                        .unwrap_or_else(|| "-".to_string());
                    let last_compaction = snapshot
                        .last_compaction_at
                        .clone()
                        .unwrap_or_else(|| "-".to_string());
                    let leaves = snapshot.leaves.clone();
                    rsx! {
                        Card {
                            CardHeader { CardTitle { "Frontier & state root" } }
                            CardContent {
                                div { class: "space-y-2 text-sm",
                                    div {
                                        span { class: "text-muted-foreground mr-2", "Frontier:" }
                                        span { class: "font-mono text-xs", "{frontier}" }
                                    }
                                    div {
                                        span { class: "text-muted-foreground mr-2", "state_root:" }
                                        span { class: "font-mono text-xs", "{state_root}" }
                                    }
                                    div {
                                        span { class: "text-muted-foreground mr-2", "Last compaction:" }
                                        span { class: "font-mono text-xs", "{last_compaction}" }
                                    }
                                }
                            }
                        }

                        Card {
                            CardHeader { CardTitle { "Anchor leaves" } }
                            CardContent {
                                div { class: "rounded-md border",
                                    Table {
                                        TableHeader {
                                            TableRow {
                                                TableHead { "Anchor ID" }
                                                TableHead { "state_root" }
                                                TableHead { "Moves" }
                                                TableHead { "Created" }
                                                TableHead { "Signers" }
                                                TableHead { "Type" }
                                            }
                                        }
                                        TableBody {
                                            if leaves.is_empty() {
                                                TableRow {
                                                    TableCell {
                                                        class: "text-center text-muted-foreground py-8".to_string(),
                                                        colspan: 99,
                                                        "No leaves visible."
                                                    }
                                                }
                                            } else {
                                                for leaf in leaves.iter() {
                                                    {
                                                        let anchor_id = leaf.anchor_id.clone();
                                                        let state_root = leaf
                                                            .state_root
                                                            .clone()
                                                            .unwrap_or_else(|| "-".to_string());
                                                        let move_count = leaf.move_count;
                                                        let created = leaf
                                                            .created_at
                                                            .clone()
                                                            .unwrap_or_else(|| "-".to_string());
                                                        let signers = leaf.signers.join(", ");
                                                        let kind_variant = if leaf.is_compaction {
                                                            BadgeVariant::Default
                                                        } else {
                                                            BadgeVariant::Secondary
                                                        };
                                                        let kind_label = if leaf.is_compaction {
                                                            "compaction"
                                                        } else {
                                                            "leaf"
                                                        };
                                                        rsx! {
                                                            TableRow {
                                                                TableCell { class: "font-mono text-xs".to_string(), "{anchor_id}" }
                                                                TableCell {
                                                                    class: "font-mono text-xs max-w-[200px] truncate".to_string(),
                                                                    "{state_root}"
                                                                }
                                                                TableCell { "{move_count}" }
                                                                TableCell { class: "text-muted-foreground".to_string(), "{created}" }
                                                                TableCell {
                                                                    class: "font-mono text-xs max-w-[260px] truncate".to_string(),
                                                                    "{signers}"
                                                                }
                                                                TableCell {
                                                                    Badge { variant: kind_variant, "{kind_label}" }
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
        }
    }
}
