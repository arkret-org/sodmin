//! Seal DAG / compaction admin page (Stream H', H'4).
//!
//! Visualises the latest Seal leaves, covered events, the latest
//! `state_root` and exposes a "trigger compaction" button that POSTs (today,
//! stub-POSTs) to soland's Seal compaction endpoint.

use dioxus::prelude::*;

use crate::api::seal;
use crate::components::selection_required::{is_placeholder_resource_id, selection_required_state};
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};

#[component]
pub fn SealDagPage(realm_id: String) -> Element {
    if is_placeholder_resource_id(&realm_id) {
        return selection_required_state("Realm");
    }

    let realm_id_for_fetch = realm_id.clone();
    let mut data = use_resource(move || {
        let id = realm_id_for_fetch.clone();
        async move { seal::get_seal_dag(&id).await }
    });

    let mut compacting = use_signal(|| false);
    let realm_id_for_compact = realm_id.clone();
    let header_realm_id = realm_id.clone();

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: format!("Seal DAG · {}", header_realm_id),
                description: "Visualize Seal leaves, covered events and state_root for this Realm.".to_string(),
                Button {
                    variant: ButtonVariant::Default,
                    disabled: *compacting.read(),
                    onclick: move |_| {
                        compacting.set(true);
                        let id = realm_id_for_compact.clone();
                        spawn(async move {
                            match seal::trigger_compaction(&id).await {
                                Ok(r) => show_toast(
                                    &format!("Compaction Seal signed: {}", r.seal_id),
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
                    // True empty snapshot: soland returned 200 but the
                    // Realm has no Seals yet. Distinguish from the
                    // error path so the operator sees "nothing to show"
                    // rather than "fetch failed".
                    if snapshot.leaves.is_empty()
                        && snapshot.covered_event_digests.is_empty()
                        && snapshot.state_root.is_none()
                    {
                        return rsx! {
                            div { class: "space-y-6",
                                EmptyState {
                                    icon_name: "shield".to_string(),
                                    title: "No Seals yet".to_string(),
                                    description: "soland returned no Seal leaves for this Realm; the DAG is empty.".to_string(),
                                }
                            }
                        };
                    }
                    let covered_events = snapshot.covered_event_digests.join(", ");
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
                            CardHeader { CardTitle { "Covered events & state root" } }
                            CardContent {
                                div { class: "space-y-2 text-sm",
                                    div {
                                        span { class: "text-muted-foreground mr-2", "Covered events:" }
                                        span { class: "font-mono text-xs", "{covered_events}" }
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
                            CardHeader { CardTitle { "Seal leaves" } }
                            CardContent {
                                div { class: "rounded-md border",
                                    Table {
                                        TableHeader {
                                            TableRow {
                                                TableHead { "Seal ID" }
                                                TableHead { "state_root" }
                                                TableHead { "Control events" }
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
                                                        let seal_id = leaf.seal_id.clone();
                                                        let state_root = leaf
                                                            .state_root
                                                            .clone()
                                                            .unwrap_or_else(|| "-".to_string());
                                                        let control_event_count = leaf.control_event_count;
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
                                                                TableCell { class: "font-mono text-xs".to_string(), "{seal_id}" }
                                                                TableCell {
                                                                    class: "font-mono text-xs max-w-[200px] truncate".to_string(),
                                                                    "{state_root}"
                                                                }
                                                                TableCell { "{control_event_count}" }
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
