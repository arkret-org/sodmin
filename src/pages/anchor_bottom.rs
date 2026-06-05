//! Bottom-state diagnostics page (Stream H', H'3).
//!
//! Lists every cell currently in `Bottom` state across visible Spaces and
//! offers a "construct repair Move" shortcut per row. Picking the action
//! pops a confirmation modal where the operator selects a typed
//! `BottomRepairStrategy` (today: `head_in_winner` only; the page falls
//! back to `manual` when no candidate heads are surfaced) before the
//! POST.

use std::collections::HashMap;

use dioxus::prelude::*;

use crate::api::anchor_admin;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::error_banner::ErrorBanner;
use crate::components::ui::loading::PageSkeleton;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::types::anchor::{BottomEntry, BottomKind, BottomRepairStrategy, WinnerHead};

#[component]
pub fn BottomDiagnosticsPage() -> Element {
    let mut data = use_resource(|| async { anchor_admin::list_bottom_entries_global().await });

    // Pending repair confirmation. `None` = modal closed; `Some` = open
    // with the entry + chosen strategy snapshot the user is about to
    // submit. We hold the strategy in the signal too so re-renders
    // during the in-flight POST don't lose the selected head.
    let mut pending = use_signal::<Option<(BottomEntry, BottomRepairStrategy)>>(|| None);
    let mut submitting = use_signal(|| false);
    // Per-row picker selection for multi-head conflict bottoms (continued
    // H'3): when `candidate_heads.len() > 1` the user picks which head to
    // promote *before* opening the confirm modal. Keyed by cell_id; absent
    // = "use default (first head)". This lets the page survive re-fetches
    // without losing in-flight selections.
    let mut selected_heads = use_signal::<HashMap<String, usize>>(HashMap::new);

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
                Some(Ok(entries)) => if entries.is_empty() {
                    rsx! {
                        EmptyState {
                            icon: "shield".to_string(),
                            title: "All clear".to_string(),
                            description: "No bottom-state cells reported across visible Spaces.".to_string(),
                        }
                    }
                } else {
                    rsx! {
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
                                            let space_id = entry.realm_id.clone();
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
                                                        {
                                                            let cell_id_for_select = entry_clone.cell_id.clone();
                                                            let cell_id_for_button = entry_clone.cell_id.clone();
                                                            let candidate_count = entry_clone.candidate_heads.len();
                                                            let current_idx = *selected_heads
                                                                .read()
                                                                .get(&cell_id_for_select)
                                                                .unwrap_or(&0usize);
                                                            let entry_for_button = entry_clone.clone();
                                                            // Inline metadata for the currently-selected
                                                            // candidate head (issuer / hlc / summary).
                                                            // soland populates these fields when
                                                            // available; the helper returns `None` when
                                                            // none are populated, so the meta block is
                                                            // skipped entirely on bare candidates.
                                                            let metadata_text = entry_clone
                                                                .candidate_heads
                                                                .get(current_idx)
                                                                .and_then(format_head_metadata);
                                                            rsx! {
                                                                div { class: "flex flex-col gap-1 items-end",
                                                                    if candidate_count > 1 {
                                                                        select {
                                                                            class: "h-8 rounded-md border bg-background px-2 text-xs",
                                                                            value: current_idx.to_string(),
                                                                            onchange: move |evt: FormEvent| {
                                                                                if let Ok(idx) = evt.value().parse::<usize>() {
                                                                                    let mut map = selected_heads.read().clone();
                                                                                    map.insert(cell_id_for_select.clone(), idx);
                                                                                    selected_heads.set(map);
                                                                                }
                                                                            },
                                                                            for (i, head) in entry_clone.candidate_heads.iter().enumerate() {
                                                                                option {
                                                                                    value: i.to_string(),
                                                                                    {format_head_option(i, head)}
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                    if let Some(meta) = metadata_text {
                                                                        div { class: "text-[10px] text-muted-foreground font-mono max-w-[280px] truncate text-right",
                                                                            "{meta}"
                                                                        }
                                                                    }
                                                                    Button {
                                                                        variant: ButtonVariant::Ghost,
                                                                        size: ButtonSize::Sm,
                                                                        onclick: move |_| {
                                                                            let idx = *selected_heads
                                                                                .read()
                                                                                .get(&cell_id_for_button)
                                                                                .unwrap_or(&0usize);
                                                                            let strategy = repair_strategy_for_entry(&entry_for_button, idx);
                                                                            pending.set(Some((entry_for_button.clone(), strategy)));
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
                                        &entry.realm_id,
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

/// Render label for a head option in the picker. Truncates the move id
/// to keep the dropdown narrow but still distinguishable. Pure helper so
/// we can unit test the formatting independent of Dioxus.
pub(crate) fn format_head_option(idx: usize, head: &WinnerHead) -> String {
    let summary = head.summary.as_deref().unwrap_or("");
    let short = if head.move_id.len() > 16 {
        format!("{}…", &head.move_id[..16])
    } else {
        head.move_id.clone()
    };
    if summary.is_empty() {
        format!("{}: {}", idx + 1, short)
    } else {
        format!("{}: {} ({})", idx + 1, short, summary)
    }
}

/// Render the inline metadata block for a selected candidate head — the
/// issuer DID, HLC timestamp and human summary fields soland may
/// populate. Returns `None` when none of the optional fields are
/// populated, so the caller can skip rendering an empty block. Pure
/// helper so the formatting logic is unit-testable.
pub(crate) fn format_head_metadata(head: &WinnerHead) -> Option<String> {
    let mut parts: Vec<String> = Vec::with_capacity(3);
    if let Some(issuer) = head.issuer.as_deref().filter(|s| !s.is_empty()) {
        parts.push(format!("issuer={issuer}"));
    }
    if let Some(hlc) = head.hlc.as_deref().filter(|s| !s.is_empty()) {
        parts.push(format!("hlc={hlc}"));
    }
    if let Some(summary) = head.summary.as_deref().filter(|s| !s.is_empty()) {
        parts.push(format!("summary={summary}"));
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" · "))
    }
}

/// Pick the repair strategy for an entry given a user-selected head
/// index. When the entry has >1 candidate heads the picker lets the
/// admin choose; we honor that selection here. Out-of-bounds indices
/// fall back to the same logic as `default_repair_strategy` so the page
/// is robust to stale `selected_heads` state across re-fetches.
pub(crate) fn repair_strategy_for_entry(
    entry: &BottomEntry,
    head_idx: usize,
) -> BottomRepairStrategy {
    if let Some(head) = entry.candidate_heads.get(head_idx) {
        match BottomKind::from_wire(&entry.kind) {
            Some(BottomKind::Conflict) | Some(BottomKind::AnchorerSplit) => {
                return BottomRepairStrategy::HeadInWinner { head: head.clone() };
            }
            _ => {}
        }
    }
    default_repair_strategy(entry)
}

/// Pick the default `BottomRepairStrategy` to seed into the confirmation
/// modal based on the bottom entry shape:
///
/// - **Conflict** with a non-empty `candidate_heads` list: pre-select the first head with
///   `HeadInWinner`. The operator confirms or backs out (and a future iteration can offer a head
///   picker before the modal opens).
/// - Otherwise (non-conflict bottoms, or conflicts with no surfaced candidates): default to
///   `Manual` with an empty effects list and a note describing the kind. soland's repair handler
///   will reject an empty manual payload, so this is intentionally a safe placeholder that fails
///   closed if the operator clicks "Submit" without first filling in effects.
pub(crate) fn default_repair_strategy(entry: &BottomEntry) -> BottomRepairStrategy {
    match (
        BottomKind::from_wire(&entry.kind),
        entry.candidate_heads.first(),
    ) {
        (Some(BottomKind::Conflict), Some(head)) => {
            BottomRepairStrategy::HeadInWinner { head: head.clone() }
        }
        (Some(BottomKind::AnchorerSplit), Some(head)) => {
            BottomRepairStrategy::HeadInWinner { head: head.clone() }
        }
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
    use super::{
        bottom_kind_variant, default_repair_strategy, format_head_metadata, format_head_option,
        format_kind_label, repair_strategy_for_entry,
    };
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
            realm_id: "ck:space:demo".into(),
            cell_id: "ck:cell:ck.component.profile.v1:ck:space:demo".into(),
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
    fn format_head_option_truncates_long_move_ids() {
        let head = WinnerHead {
            move_id: "sha256:aaaabbbbccccddddeeeeffff".into(),
            ..Default::default()
        };
        let label = format_head_option(0, &head);
        // 1-indexed, truncated with ellipsis at 16 chars of the move id.
        assert!(label.starts_with("1: "));
        assert!(label.contains("\u{2026}"));
        assert!(!label.contains("ffff"));
    }

    #[test]
    fn format_head_option_includes_summary_when_present() {
        let head = WinnerHead {
            move_id: "m:abc".into(),
            summary: Some("set value=42".into()),
            ..Default::default()
        };
        let label = format_head_option(2, &head);
        assert!(label.starts_with("3: "));
        assert!(label.contains("m:abc"));
        assert!(label.contains("set value=42"));
    }

    #[test]
    fn repair_strategy_picker_uses_chosen_index() {
        // Conflict bottom with 3 candidate heads → picker index 2 should
        // produce HeadInWinner { head: heads[2] }, not the default first.
        let entry = BottomEntry {
            realm_id: "ck:space:demo".into(),
            cell_id: "ck:cell:ck.component.profile.v1:ck:space:demo".into(),
            kind: "conflict".into(),
            candidate_heads: vec![
                WinnerHead {
                    move_id: "m:1".into(),
                    ..Default::default()
                },
                WinnerHead {
                    move_id: "m:2".into(),
                    ..Default::default()
                },
                WinnerHead {
                    move_id: "m:3".into(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        match repair_strategy_for_entry(&entry, 2) {
            BottomRepairStrategy::HeadInWinner { head } => assert_eq!(head.move_id, "m:3"),
            other => panic!("expected head_in_winner with picked index, got {other:?}"),
        }
    }

    #[test]
    fn repair_strategy_picker_falls_back_when_index_out_of_bounds() {
        // Out-of-bounds index → fall back to default_repair_strategy
        // (which picks the first head for a conflict entry).
        let entry = BottomEntry {
            realm_id: "ck:space:demo".into(),
            cell_id: "ck:cell:ck.component.profile.v1:ck:space:demo".into(),
            kind: "conflict".into(),
            candidate_heads: vec![WinnerHead {
                move_id: "m:first".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        match repair_strategy_for_entry(&entry, 99) {
            BottomRepairStrategy::HeadInWinner { head } => assert_eq!(head.move_id, "m:first"),
            other => panic!("expected default head_in_winner fallback, got {other:?}"),
        }
    }

    #[test]
    fn head_metadata_is_none_when_no_optional_fields_populated() {
        let head = WinnerHead {
            move_id: "m:1".into(),
            ..Default::default()
        };
        assert!(format_head_metadata(&head).is_none());
    }

    #[test]
    fn head_metadata_concatenates_populated_optional_fields() {
        // All three populated: ordering is issuer · hlc · summary so the
        // operator gets a stable, predictable line.
        let head = WinnerHead {
            move_id: "m:1".into(),
            issuer: Some("did:ck:alice".into()),
            hlc: Some("01J9-0001-abcd".into()),
            summary: Some("set value=42".into()),
        };
        let meta = format_head_metadata(&head).expect("metadata present");
        assert!(meta.starts_with("issuer=did:ck:alice"));
        assert!(meta.contains("hlc=01J9-0001-abcd"));
        assert!(meta.ends_with("summary=set value=42"));
        // Empty-string optional fields are treated as absent — soland
        // sometimes serializes "" instead of `null` and we must not show
        // a bare "issuer=" key.
        let head = WinnerHead {
            move_id: "m:1".into(),
            issuer: Some(String::new()),
            hlc: None,
            summary: Some("only this".into()),
        };
        let meta = format_head_metadata(&head).expect("metadata present");
        assert!(!meta.contains("issuer="));
        assert!(meta.contains("summary=only this"));
    }

    #[test]
    fn default_strategy_falls_back_to_manual_when_no_candidates() {
        // Non-conflict bottom kind with no candidate heads → manual.
        let entry = BottomEntry {
            realm_id: "ck:space:demo".into(),
            cell_id: "ck:cell:ck.component.member.state.v1:did:ck:alice".into(),
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
            realm_id: "ck:space:demo".into(),
            cell_id: "ck:cell:ck.component.profile.v1:ck:space:demo".into(),
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
