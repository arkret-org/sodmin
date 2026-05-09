//! Bottom-state diagnostics page (Stream H', H'3).
//!
//! Lists every cell currently in `Bottom` state across visible Spaces and
//! offers a "construct repair Move" shortcut per row. Picking the action
//! pops a confirmation modal where the operator selects a typed
//! `BottomRepairStrategy` (today: `head_in_winner` only; the page falls
//! back to `manual` when no candidate heads are surfaced) before the
//! POST.

use dioxus::prelude::*;

use crate::api::anchor_admin;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::anchor::{BottomEntry, BottomKind, BottomRepairStrategy};

#[component]
pub fn BottomDiagnosticsPage() -> Element {
    let mut data = use_resource(|| async { anchor_admin::list_bottom_entries_global().await });

    // Pending repair confirmation. `None` = modal closed; `Some` = open
    // with the entry + chosen strategy snapshot the user is about to
    // submit. We hold the strategy in the signal too so re-renders
    // during the in-flight POST don't lose the selected head.
    let mut pending = use_signal::<Option<(BottomEntry, BottomRepairStrategy)>>(|| None);
    let mut submitting = use_signal(|| false);

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
                                            let entry_clone = entry.clone();
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
                                            let space_id = entry.space_id.clone();
                                            let cell_id = entry.cell_id.clone();
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
                                                                let strategy = default_repair_strategy(&entry_clone);
                                                                pending.set(Some((entry_clone.clone(), strategy)));
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

            // Confirmation modal. We render the dialog whenever
            // `pending` is `Some`; the dialog itself short-circuits when
            // `open=false`, so it's always cheap.
            {
                let (entry_opt, strategy_opt) = match &*pending.read() {
                    Some((e, s)) => (Some(e.clone()), Some(s.clone())),
                    None => (None, None),
                };
                let open = entry_opt.is_some();
                let title = "Submit repair Move?".to_string();
                let description = match (&entry_opt, &strategy_opt) {
                    (Some(e), Some(s)) => format!(
                        "Cell {} ({}); strategy = {}.",
                        e.cell_id,
                        format_kind_label(&e.kind),
                        s.label()
                    ),
                    _ => String::new(),
                };
                let confirm_text = if *submitting.read() {
                    "Submitting…".to_string()
                } else {
                    "Submit".to_string()
                };
                rsx! {
                    ConfirmDialog {
                        open,
                        title,
                        description,
                        confirm_text,
                        cancel_text: "Cancel".to_string(),
                        destructive: true,
                        on_cancel: move |_| pending.set(None),
                        on_confirm: move |_| {
                            if *submitting.read() { return; }
                            let snapshot = pending.read().clone();
                            if let Some((entry, strategy)) = snapshot {
                                submitting.set(true);
                                spawn(async move {
                                    let res = anchor_admin::submit_bottom_repair(
                                        &entry.space_id,
                                        &entry.cell_id,
                                        strategy,
                                    )
                                    .await;
                                    match res {
                                        Ok(r) => show_toast(
                                            &format!("Repair move: {}", r.move_id),
                                            ToastVariant::Success,
                                        ),
                                        Err(e) => show_toast(
                                            &format!("Failed: {}", e.message),
                                            ToastVariant::Error,
                                        ),
                                    }
                                    submitting.set(false);
                                    pending.set(None);
                                    data.restart();
                                });
                            }
                        },
                    }
                }
            }
        }
    }
}

/// Pick the default `BottomRepairStrategy` to seed into the confirmation
/// modal based on the bottom entry shape:
///
/// - **Conflict** with a non-empty `candidate_heads` list: pre-select
///   the first head with `HeadInWinner`. The operator confirms or backs
///   out (and a future iteration can offer a head picker before the
///   modal opens).
/// - Otherwise (non-conflict bottoms, or conflicts with no surfaced
///   candidates): default to `Manual` with an empty effects list and a
///   note describing the kind. soland's repair handler will reject an
///   empty manual payload, so this is intentionally a safe placeholder
///   that fails closed if the operator clicks "Submit" without first
///   filling in effects.
pub(crate) fn default_repair_strategy(entry: &BottomEntry) -> BottomRepairStrategy {
    match (BottomKind::from_wire(&entry.kind), entry.candidate_heads.first()) {
        (Some(BottomKind::Conflict), Some(head)) => BottomRepairStrategy::HeadInWinner {
            head: head.clone(),
        },
        (Some(BottomKind::AnchorerSplit), Some(head)) => BottomRepairStrategy::HeadInWinner {
            head: head.clone(),
        },
        _ => BottomRepairStrategy::Manual {
            note: Some(format!(
                "Manual repair — bottom kind = {}",
                format_kind_label(&entry.kind)
            )),
            effects: vec![],
        },
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
    use super::{bottom_kind_variant, default_repair_strategy, format_kind_label};
    use crate::components::ui::badge::BadgeVariant;
    use crate::types::anchor::{BottomEntry, BottomRepairStrategy, WinnerHead};

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

    #[test]
    fn default_strategy_picks_head_in_for_conflict_with_candidates() {
        let entry = BottomEntry {
            space_id: "cx:space:demo".into(),
            cell_id: "cx:cell:cx.component.profile.v1:cx:space:demo".into(),
            kind: "conflict".into(),
            candidate_heads: vec![WinnerHead {
                move_id: "move:abc".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        match default_repair_strategy(&entry) {
            BottomRepairStrategy::HeadInWinner { head } => {
                assert_eq!(head.move_id, "move:abc");
            }
            other => panic!("expected head_in_winner default, got {other:?}"),
        }
    }

    #[test]
    fn default_strategy_falls_back_to_manual_when_no_candidates() {
        // Non-conflict bottom kind with no candidate heads → manual.
        let entry = BottomEntry {
            space_id: "cx:space:demo".into(),
            cell_id: "cx:cell:cx.component.member.state.v1:did:cx:alice".into(),
            kind: "schema_error".into(),
            ..Default::default()
        };
        match default_repair_strategy(&entry) {
            BottomRepairStrategy::Manual { note, effects } => {
                assert!(effects.is_empty());
                assert!(note.unwrap_or_default().contains("Schema Error"));
            }
            other => panic!("expected manual default, got {other:?}"),
        }

        // Conflict but candidate_heads empty → still manual (operator
        // must hand-craft because nothing to pick).
        let entry = BottomEntry {
            space_id: "cx:space:demo".into(),
            cell_id: "cx:cell:cx.component.profile.v1:cx:space:demo".into(),
            kind: "conflict".into(),
            candidate_heads: vec![],
            ..Default::default()
        };
        assert!(matches!(
            default_repair_strategy(&entry),
            BottomRepairStrategy::Manual { .. }
        ));
    }
}
