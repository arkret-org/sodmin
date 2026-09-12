//! Read-only diagnostics for ordinary causal-register conflicts.
//! Candidate values do not grant authority or define a generic recovery operation.

use dioxus::prelude::*;

use crate::api::seal;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonVariant};
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::types::seal::{BottomKindExt, bottom_kind_from_wire};
use crate::utils::i18n::t;

#[component]
pub fn BottomDiagnosticsPage() -> Element {
    let mut data = use_resource(|| async { seal::list_bottom_entries_global().await });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: t("seal_bottom.title"),
                description: t("seal_bottom.description"),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    {t("common.refresh")}
                }
            }
            match &*data.read() {
                Some(Ok(entries)) if entries.is_empty() => rsx! {
                    EmptyState {
                        icon_name: "shield".to_string(),
                        title: t("seal_bottom.all_clear"),
                        description: t("seal_bottom.all_clear_description"),
                    }
                },
                Some(Ok(entries)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { {t("seal_bottom.col_kind")} }
                                    TableHead { {t("seal_bottom.col_realm")} }
                                    TableHead { {t("seal_bottom.col_cell")} }
                                    TableHead { {t("seal_bottom.col_event_ids")} }
                                    TableHead { {t("seal_bottom.col_details")} }
                                }
                            }
                            TableBody {
                                for entry in entries {
                                    TableRow {
                                        TableCell {
                                            Badge {
                                                variant: BadgeVariant::Destructive,
                                                {bottom_kind_from_wire(&entry.kind).map(|kind| kind.label().to_owned()).unwrap_or_else(|| entry.kind.clone())}
                                            }
                                        }
                                        TableCell { class: "font-mono text-xs", "{entry.realm_id}" }
                                        TableCell { class: "font-mono text-xs break-all", "{entry.cell_id}" }
                                        TableCell {
                                            for head in &entry.candidate_heads {
                                                div { class: "font-mono text-xs break-all", "{head.event_id}" }
                                            }
                                        }
                                        TableCell {
                                            for head in &entry.candidate_heads {
                                                pre { class: "text-xs whitespace-pre-wrap break-all", "{head.value}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                Some(Err(error)) => rsx! {
                    ErrorBanner {
                        message: error.message.clone(),
                        on_retry: move |_| data.restart(),
                    }
                },
                None => rsx! { PageSkeleton {} },
            }
        }
    }
}
