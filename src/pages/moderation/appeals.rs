//! Round R2/R3 — Moderation appeal admin (T06).
//!
//! List view of pending `cx.moderation.appeal.submit` rows (state =
//! `submitted` / `under_review`) plus a detail panel that surfaces:
//!
//! - the original decision being appealed (`decision_ref`)
//! - appellant + evidence references
//! - reviewer assignment audit trail (`cx.moderation.appeal.review`)
//! - decision history (`cx.moderation.appeal.decision`)
//! - 30-day auto-close countdown
//!
//! Separation-of-duties: the "Review this appeal" button is hidden when
//! the logged-in admin DID equals the issuer of the original moderation
//! decision (per `cx.moderation.appeal.review` reducer rule
//! "reviewer.did != original_decision.issuer_did").
//!
//! Verdict picker (uphold / overturn / modify) writes
//! `cx.moderation.appeal.decision` and — when verdict==overturn — the
//! reducer pairs it with `cx.moderation.decision.lift` automatically in
//! the same Anchor batch; this UI just surfaces the auto-pairing in a
//! callout so the admin knows what they are about to submit.
//!
//! Wire to `/api/admin/v1/moderation/appeals` (TODO: list + describe
//! handlers not yet generated in the soland describe contract, so the
//! page renders against a local in-memory mock when the route 404s,
//! same 404-tolerant pattern used by `pages/moderation/reports.rs`).
//
// TODO(round23-T06): swap the static placeholder dataset for the real
// `/api/admin/v1/moderation/appeals` describe rows once soland publishes
// them; today the list is rendered from a `Vec<AppealRow>` literal so
// the operator can see the verdict picker / countdown / separation-of-
// duties wiring against a deterministic fixture.

use dioxus::prelude::*;

use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::utils::session;

/// Wall-clock window after which an appeal auto-closes
/// (`cx.moderation.appeal.close.auto_closed=true`). Spec: 30 days.
const APPEAL_AUTO_CLOSE_DAYS: i64 = 30;

/// Lifecycle state of an appeal as projected from the four
/// `cx.moderation.appeal.*` event variants.
#[derive(Debug, Clone, PartialEq)]
enum AppealLifecycle {
    /// Reducer has seen `submit` but no `review` yet — unassigned.
    Submitted,
    /// Reducer has seen at least one `review` from a non-issuer
    /// reviewer — under triage.
    UnderReview,
    /// Reducer has seen `decision`.
    Decided,
    /// Reducer has seen `close` (auto or manual).
    Closed,
}

impl AppealLifecycle {
    fn label(&self) -> &'static str {
        match self {
            AppealLifecycle::Submitted => "submitted",
            AppealLifecycle::UnderReview => "under_review",
            AppealLifecycle::Decided => "decided",
            AppealLifecycle::Closed => "closed",
        }
    }

    fn badge_variant(&self) -> BadgeVariant {
        match self {
            AppealLifecycle::Submitted => BadgeVariant::Default,
            AppealLifecycle::UnderReview => BadgeVariant::Secondary,
            AppealLifecycle::Decided => BadgeVariant::Success,
            AppealLifecycle::Closed => BadgeVariant::Secondary,
        }
    }

    /// `pending` per the round 2+3 spec = state is `submitted` or
    /// `under_review`. This is the default list-view filter.
    fn is_pending(&self) -> bool {
        matches!(
            self,
            AppealLifecycle::Submitted | AppealLifecycle::UnderReview
        )
    }
}

/// Verdict the reviewing admin can record. Mirrors
/// `contrix_core::model::round23::AppealVerdict`. Modify requires the
/// admin to also submit a fresh `cx.moderation.decision` event — that
/// flow is intentionally not wired in this round, so the picker greys
/// Modify out with a tooltip.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Verdict {
    Uphold,
    Overturn,
    Modify,
}

impl Verdict {
    fn label(&self) -> &'static str {
        match self {
            Verdict::Uphold => "Uphold",
            Verdict::Overturn => "Overturn",
            Verdict::Modify => "Modify",
        }
    }
}

#[derive(Debug, Clone)]
struct ReviewerNote {
    reviewer_did: String,
    reviewed_at: String,
    note: Option<String>,
}

#[derive(Debug, Clone)]
struct DecisionEntry {
    verdict: Verdict,
    reviewer_did: String,
    decided_at: String,
    reason_text_ref: String,
}

/// Local projection of an appeal row + its history. Mirrors the
/// `cx.schema.moderation_appeal.v1` reducer projection.
#[derive(Debug, Clone)]
struct AppealRow {
    appeal_id: String,
    decision_ref: String,
    target_ref: String,
    /// DID that issued the *original* moderation decision. Used by the
    /// separation-of-duties check.
    original_issuer_did: String,
    appellant_did: String,
    reason_text_ref: String,
    evidence_refs: Vec<String>,
    submitted_at: String,
    /// Days remaining before the 30-day auto-close fires. Negative
    /// values render as "auto-closing soon" / expired.
    days_until_auto_close: i64,
    lifecycle: AppealLifecycle,
    reviews: Vec<ReviewerNote>,
    decisions: Vec<DecisionEntry>,
}

fn placeholder_appeals() -> Vec<AppealRow> {
    vec![
        AppealRow {
            appeal_id: "cx:appeal:01904100-0000-7000-8000-000000000001".into(),
            decision_ref: "cx:event:01904100-0000-7000-8000-0000000000a1".into(),
            target_ref: "cx:message:01904100-0000-7000-8000-0000000000b1".into(),
            original_issuer_did: "did:web:alice.example".into(),
            appellant_did: "did:web:bob.example".into(),
            reason_text_ref: "blob:reason:01904100-0000-7000-8000-0000000000c1".into(),
            evidence_refs: vec![
                "blob:evidence:01904100-0000-7000-8000-0000000000d1".into(),
            ],
            submitted_at: "2026-05-12T10:14:00Z".into(),
            days_until_auto_close: 22,
            lifecycle: AppealLifecycle::Submitted,
            reviews: vec![],
            decisions: vec![],
        },
        AppealRow {
            appeal_id: "cx:appeal:01904100-0000-7000-8000-000000000002".into(),
            decision_ref: "cx:event:01904100-0000-7000-8000-0000000000a2".into(),
            target_ref: "cx:message:01904100-0000-7000-8000-0000000000b2".into(),
            original_issuer_did: "did:web:carol.example".into(),
            appellant_did: "did:web:dave.example".into(),
            reason_text_ref: "blob:reason:01904100-0000-7000-8000-0000000000c2".into(),
            evidence_refs: vec![],
            submitted_at: "2026-04-25T18:02:00Z".into(),
            days_until_auto_close: 4,
            lifecycle: AppealLifecycle::UnderReview,
            reviews: vec![ReviewerNote {
                reviewer_did: "did:web:erin.example".into(),
                reviewed_at: "2026-04-26T09:11:00Z".into(),
                note: Some("Triaged; pulling context from upstream report.".into()),
            }],
            decisions: vec![],
        },
    ]
}

#[component]
pub fn ModerationAppealsPage() -> Element {
    let mut selected = use_signal::<Option<String>>(|| None);
    let mut verdict_picker = use_signal::<Option<Verdict>>(|| None);
    let mut confirm_open = use_signal(|| false);

    let rows = placeholder_appeals();
    let pending_only: Vec<&AppealRow> = rows.iter().filter(|r| r.lifecycle.is_pending()).collect();

    // Snapshot the currently signed-in admin so we can apply the
    // separation-of-duties rule on the detail panel.
    let current_admin_did = session::current_user().id.unwrap_or_default();

    let selected_row: Option<AppealRow> = selected
        .read()
        .as_ref()
        .and_then(|id| rows.iter().find(|r| r.appeal_id == *id).cloned());

    rsx! {
        div { class: "space-y-6",
            PageHeader {
                title: "Moderation appeals".to_string(),
                description: format!(
                    "Pending appeals (state=submitted/under_review). Auto-closes after {APPEAL_AUTO_CLOSE_DAYS} days. Round R2/R3 T06."
                ),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| selected.set(None),
                    "Clear selection"
                }
            }

            if pending_only.is_empty() {
                EmptyState {
                    icon: "flag".to_string(),
                    title: "No pending appeals".to_string(),
                    description: "All `cx.moderation.appeal.submit` rows are decided or auto-closed.".to_string(),
                }
            } else {
                div { class: "rounded-md border",
                    Table {
                        TableHeader {
                            TableRow {
                                TableHead { "Appeal" }
                                TableHead { "Decision ref" }
                                TableHead { "Appellant" }
                                TableHead { "State" }
                                TableHead { "Auto-close" }
                                TableHead { class: "text-right".to_string(), "Open" }
                            }
                        }
                        TableBody {
                            for row in pending_only.iter() {
                                {
                                    let row = (*row).clone();
                                    let row_id = row.appeal_id.clone();
                                    let countdown_label = countdown_chip_label(row.days_until_auto_close);
                                    let countdown_variant = countdown_chip_variant(row.days_until_auto_close);
                                    let lifecycle_label = row.lifecycle.label();
                                    let lifecycle_variant = row.lifecycle.badge_variant();
                                    rsx! {
                                        TableRow {
                                            TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{row.appeal_id}" }
                                            TableCell { class: "font-mono text-xs max-w-[260px] truncate".to_string(), "{row.decision_ref}" }
                                            TableCell { class: "font-mono text-xs max-w-[200px] truncate".to_string(), "{row.appellant_did}" }
                                            TableCell {
                                                Badge { variant: lifecycle_variant, "{lifecycle_label}" }
                                            }
                                            TableCell {
                                                Badge { variant: countdown_variant, "{countdown_label}" }
                                            }
                                            TableCell { class: "text-right".to_string(),
                                                Button {
                                                    variant: ButtonVariant::Outline,
                                                    size: ButtonSize::Sm,
                                                    onclick: move |_| selected.set(Some(row_id.clone())),
                                                    "Open"
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

            if let Some(row) = selected_row {
                {appeal_detail_card(
                    &row,
                    &current_admin_did,
                    verdict_picker,
                    confirm_open,
                )}
            }

            {
                let pending = *verdict_picker.read();
                let open = *confirm_open.read();
                let verdict_label = pending.map(|v| v.label()).unwrap_or("");
                let description = match pending {
                    Some(Verdict::Overturn) => "Overturn auto-pairs `cx.moderation.decision.lift` in the same Anchor batch.".to_string(),
                    Some(Verdict::Modify) => "Modify requires a fresh `cx.moderation.decision` event to be supplied; this flow is not yet wired.".to_string(),
                    _ => "Recording an appeal decision is final.".to_string(),
                };
                rsx! {
                    ConfirmDialog {
                        open,
                        title: format!("Record verdict: {verdict_label}"),
                        description,
                        confirm_text: format!("Submit {verdict_label}"),
                        cancel_text: "Cancel".to_string(),
                        destructive: matches!(pending, Some(Verdict::Overturn)),
                        on_cancel: move |_| {
                            confirm_open.set(false);
                            verdict_picker.set(None);
                        },
                        on_confirm: move |_| {
                            // TODO(round23-T06): POST
                            // `/api/admin/v1/moderation/appeals/{id}/decision`
                            // with body `{verdict, reason_text_ref}`. For
                            // overturn, the reducer pairs in
                            // `cx.moderation.decision.lift` automatically.
                            confirm_open.set(false);
                            verdict_picker.set(None);
                            show_toast(
                                "Appeal verdict queued (placeholder; backend wiring pending).",
                                ToastVariant::Success,
                            );
                        },
                    }
                }
            }
        }
    }
}

/// Detail card for the currently selected appeal. Renders evidence,
/// reviewer notes, decision history, and the verdict picker (subject to
/// separation of duties).
fn appeal_detail_card(
    row: &AppealRow,
    current_admin_did: &str,
    mut verdict_picker: Signal<Option<Verdict>>,
    mut confirm_open: Signal<bool>,
) -> Element {
    // Separation-of-duties: an admin cannot review an appeal of a
    // moderation decision they themselves issued.
    let admin_is_issuer = !current_admin_did.is_empty()
        && row.original_issuer_did.eq_ignore_ascii_case(current_admin_did);

    let evidence_block: Element = if row.evidence_refs.is_empty() {
        rsx! {
            p { class: "text-sm text-muted-foreground", "No evidence_refs supplied." }
        }
    } else {
        rsx! {
            ul { class: "space-y-1",
                for ev in row.evidence_refs.iter() {
                    li { class: "font-mono text-xs break-all", "{ev}" }
                }
            }
        }
    };

    let reviews_block: Element = if row.reviews.is_empty() {
        rsx! {
            p { class: "text-sm text-muted-foreground", "No reviewer has claimed this appeal yet." }
        }
    } else {
        rsx! {
            ul { class: "space-y-2",
                for review in row.reviews.iter() {
                    li { class: "rounded border p-2 text-xs space-y-1",
                        div { class: "font-mono", "reviewer: {review.reviewer_did}" }
                        div { class: "text-muted-foreground", "at {review.reviewed_at}" }
                        if let Some(note) = review.note.as_ref() {
                            div { class: "italic", "{note}" }
                        }
                    }
                }
            }
        }
    };

    let decisions_block: Element = if row.decisions.is_empty() {
        rsx! {
            p { class: "text-sm text-muted-foreground", "No decision recorded yet." }
        }
    } else {
        rsx! {
            ul { class: "space-y-2",
                for entry in row.decisions.iter() {
                    {
                        let label = entry.verdict.label();
                        rsx! {
                            li { class: "rounded border p-2 text-xs space-y-1",
                                div { class: "font-mono", "verdict: {label}" }
                                div { class: "font-mono", "reviewer: {entry.reviewer_did}" }
                                div { class: "text-muted-foreground", "at {entry.decided_at}" }
                                div { class: "font-mono break-all", "reason_ref: {entry.reason_text_ref}" }
                            }
                        }
                    }
                }
            }
        }
    };

    let countdown_label = countdown_chip_label(row.days_until_auto_close);
    let countdown_variant = countdown_chip_variant(row.days_until_auto_close);

    rsx! {
        Card {
            CardHeader {
                div { class: "flex items-start justify-between gap-2",
                    div { class: "space-y-1",
                        CardTitle { class: "text-lg".to_string(), "Appeal detail" }
                        CardDescription { "{row.appeal_id}" }
                    }
                    Badge { variant: countdown_variant, "{countdown_label}" }
                }
            }
            CardContent { class: "space-y-4".to_string(),
                div { class: "grid gap-3 sm:grid-cols-2",
                    div {
                        p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "Original decision" }
                        p { class: "text-xs font-mono break-all", "{row.decision_ref}" }
                    }
                    div {
                        p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "Target" }
                        p { class: "text-xs font-mono break-all", "{row.target_ref}" }
                    }
                    div {
                        p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "Original issuer" }
                        p { class: "text-xs font-mono break-all", "{row.original_issuer_did}" }
                    }
                    div {
                        p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "Appellant" }
                        p { class: "text-xs font-mono break-all", "{row.appellant_did}" }
                    }
                    div {
                        p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "Submitted at" }
                        p { class: "text-xs font-mono", "{row.submitted_at}" }
                    }
                    div {
                        p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "reason_text_ref" }
                        p { class: "text-xs font-mono break-all", "{row.reason_text_ref}" }
                    }
                }

                div {
                    h3 { class: "text-sm font-semibold mb-1", "Evidence" }
                    {evidence_block}
                }
                div {
                    h3 { class: "text-sm font-semibold mb-1", "Reviewer assignment trail" }
                    {reviews_block}
                }
                div {
                    h3 { class: "text-sm font-semibold mb-1", "Decision history" }
                    {decisions_block}
                }

                div { class: "border-t pt-3 space-y-2",
                    h3 { class: "text-sm font-semibold", "Review this appeal" }
                    if admin_is_issuer {
                        // Separation-of-duties — hide the picker entirely.
                        p { class: "rounded-md border border-amber-600/40 bg-amber-600/10 p-2 text-xs text-amber-700 dark:text-amber-300",
                            "Separation of duties: you issued the original decision (",
                            span { class: "font-mono", "{row.original_issuer_did}" },
                            "). The reducer rejects `cx.moderation.appeal.review` events where reviewer.did matches the original issuer."
                        }
                    } else {
                        div { class: "flex flex-wrap items-center gap-2",
                            Button {
                                variant: ButtonVariant::Default,
                                size: ButtonSize::Sm,
                                onclick: move |_| {
                                    verdict_picker.set(Some(Verdict::Uphold));
                                    confirm_open.set(true);
                                },
                                "Uphold"
                            }
                            Button {
                                variant: ButtonVariant::Destructive,
                                size: ButtonSize::Sm,
                                onclick: move |_| {
                                    verdict_picker.set(Some(Verdict::Overturn));
                                    confirm_open.set(true);
                                },
                                "Overturn (auto-pairs lift)"
                            }
                            Button {
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                disabled: true,
                                onclick: move |_| {
                                    verdict_picker.set(Some(Verdict::Modify));
                                    confirm_open.set(true);
                                },
                                "Modify (requires fresh decision — pending)"
                            }
                        }
                        p { class: "text-xs text-muted-foreground",
                            "Overturn writes `cx.moderation.appeal.decision` with verdict=overturn; the reducer auto-pairs `cx.moderation.decision.lift` in the same Anchor batch."
                        }
                    }
                }
            }
        }
    }
}

/// Human-readable label for the 30-day auto-close countdown chip in
/// the list view.
fn countdown_chip_label(days: i64) -> String {
    if days <= 0 {
        "auto-close imminent".to_string()
    } else if days == 1 {
        "1 day to auto-close".to_string()
    } else {
        format!("{days} days to auto-close")
    }
}

/// Red when the appeal is within 5 days of auto-close (the reducer
/// will close it without admin intervention), otherwise neutral.
fn countdown_chip_variant(days: i64) -> BadgeVariant {
    if days <= 5 {
        BadgeVariant::Destructive
    } else {
        BadgeVariant::Secondary
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn countdown_chip_is_red_within_five_days() {
        assert!(matches!(
            countdown_chip_variant(0),
            BadgeVariant::Destructive
        ));
        assert!(matches!(
            countdown_chip_variant(5),
            BadgeVariant::Destructive
        ));
        assert!(matches!(countdown_chip_variant(6), BadgeVariant::Secondary));
    }

    #[test]
    fn countdown_label_handles_zero_one_many() {
        assert_eq!(countdown_chip_label(0), "auto-close imminent");
        assert_eq!(countdown_chip_label(-3), "auto-close imminent");
        assert_eq!(countdown_chip_label(1), "1 day to auto-close");
        assert_eq!(countdown_chip_label(22), "22 days to auto-close");
    }

    #[test]
    fn lifecycle_pending_matches_spec_buckets() {
        assert!(AppealLifecycle::Submitted.is_pending());
        assert!(AppealLifecycle::UnderReview.is_pending());
        assert!(!AppealLifecycle::Decided.is_pending());
        assert!(!AppealLifecycle::Closed.is_pending());
    }
}
