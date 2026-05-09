//! Bottom-state diagnostics page (Stream H', H'3).
//!
//! Lists every cell currently in `Bottom` state across visible Spaces and
//! offers a "construct repair Move" shortcut per row. Rendering only — the
//! data fetch and repair-move submission both go through stubs in
//! `anchor_admin` until the soland MAL-5 describe endpoint stabilises.

use dioxus::prelude::*;

use crate::api::anchor_admin;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::anchor::BottomKind;

#[component]
pub fn BottomDiagnosticsPage() -> Element {
    // TODO(soland-admin-api): replace stub fetch with bottom-diagnostics
    // describe endpoint once soland MAL-5 lands.
    let mut data = use_resource(|| async { anchor_admin::list_bottom_entries().await });

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Bottom diagnostics".to_string(),
                description: "Cells currently in Bottom state across visible Spaces.".to_string(),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| data.restart(),
                    "Refresh"
                }
            }

            match &*data.read() {
                Some(Ok(entries)) => rsx! {
                    div { class: "rounded-md border",
                        Table {
                            TableHeader {
                                TableRow {
                                    TableHead { "Kind" }
                                    TableHead { "Space" }
                                    TableHead { "Cell" }
                                    TableHead { "Move IDs" }
                                    TableHead { "Detected" }
                                    TableHead { "Details" }
                                    TableHead { class: "text-right".to_string(), "Action" }
                                }
                            }
                            TableBody {
                                if entries.is_empty() {
                                    TableRow {
                                        TableCell {
                                            class: "text-center text-muted-foreground py-8".to_string(),
                                            colspan: 99,
                                            "No bottom-state cells reported."
                                        }
                                    }
                                } else {
                                    for entry in entries.iter() {
                                        {
                                            let space_id = entry.space_id.clone();
                                            let cell_id = entry.cell_id.clone();
                                            let kind_label = format_kind_label(&entry.kind);
                                            let kind_variant = bottom_kind_variant(&entry.kind);
                                            let move_ids = entry.move_ids.join(", ");
                                            let detected = entry
                                                .detected_at
                                                .clone()
                                                .unwrap_or_else(|| "-".to_string());
                                            let details = entry
                                                .details
                                                .clone()
                                                .unwrap_or_else(|| "-".to_string());
                                            let action_space = space_id.clone();
                                            let action_cell = cell_id.clone();
                                            rsx! {
                                                TableRow {
                                                    TableCell {
                                                        Badge { variant: kind_variant, "{kind_label}" }
                                                    }
                                                    TableCell { class: "font-mono text-xs".to_string(), "{space_id}" }
                                                    TableCell {
                                                        class: "font-mono text-xs max-w-[260px] truncate".to_string(),
                                                        "{cell_id}"
                                                    }
                                                    TableCell {
                                                        class: "font-mono text-xs max-w-[200px] truncate".to_string(),
                                                        "{move_ids}"
                                                    }
                                                    TableCell { class: "text-muted-foreground".to_string(), "{detected}" }
                                                    TableCell {
                                                        class: "max-w-[280px] truncate".to_string(),
                                                        "{details}"
                                                    }
                                                    TableCell { class: "text-right".to_string(),
                                                        Button {
                                                            variant: ButtonVariant::Ghost,
                                                            size: ButtonSize::Sm,
                                                            onclick: move |_| {
                                                                let s = action_space.clone();
                                                                let c = action_cell.clone();
                                                                spawn(async move {
                                                                    // TODO(soland-admin-api): build a real
                                                                    // repair Move and submit to /api/v1/moves.
                                                                    match anchor_admin::submit_bottom_repair(&s, &c).await {
                                                                        Ok(r) => show_toast(
                                                                            &format!("Repair move: {}", r.move_id),
                                                                            ToastVariant::Success,
                                                                        ),
                                                                        Err(e) => show_toast(
                                                                            &format!("Failed: {}", e.message),
                                                                            ToastVariant::Error,
                                                                        ),
                                                                    }
                                                                });
                                                            },
                                                            "Construct repair Move"
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

pub(crate) fn format_kind_label(wire: &str) -> String {
    BottomKind::from_wire(wire)
        .map(|k| k.label().to_string())
        .unwrap_or_else(|| wire.to_string())
}

pub(crate) fn bottom_kind_variant(wire: &str) -> BadgeVariant {
    match BottomKind::from_wire(wire) {
        Some(BottomKind::Conflict) | Some(BottomKind::AnchorerSplit) => BadgeVariant::Destructive,
        Some(BottomKind::Unauthorized) | Some(BottomKind::SchemaError) => BadgeVariant::Destructive,
        Some(BottomKind::InvalidTransition) | Some(BottomKind::MissingDependency) => {
            BadgeVariant::Secondary
        }
        None => BadgeVariant::Outline,
    }
}

#[cfg(test)]
mod tests {
    use super::{bottom_kind_variant, format_kind_label};
    use crate::components::ui::badge::BadgeVariant;

    #[test]
    fn format_kind_label_falls_back_to_raw() {
        assert_eq!(format_kind_label("conflict"), "Conflict");
        assert_eq!(format_kind_label("anchorer_split"), "Anchorer Split");
        // Unknown wire value falls back to the raw string so admins see
        // SOMETHING rather than an empty cell.
        assert_eq!(format_kind_label("never_seen"), "never_seen");
    }

    #[test]
    fn bottom_kind_variant_buckets() {
        assert!(matches!(
            bottom_kind_variant("conflict"),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            bottom_kind_variant("anchorer_split"),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            bottom_kind_variant("missing_dependency"),
            BadgeVariant::Secondary
        ));
        assert!(matches!(
            bottom_kind_variant("schema_error"),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            bottom_kind_variant("garbage"),
            BadgeVariant::Outline
        ));
    }
}
