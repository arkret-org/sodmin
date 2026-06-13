//! Moderation appeal admin.
//!
//! List view of pending `ck.moderation.appeal.submit` rows (state =
//! `submitted` / `under_review`) plus a detail panel that surfaces:
//!
//! - the original decision being appealed (`decision_ref`)
//! - appellant + evidence references
//! - reviewer assignment audit trail (`ck.moderation.appeal.review`)
//! - decision history (`ck.moderation.appeal.decision`)
//! - 30-day auto-close countdown
//!
//! Separation-of-duties: the "Review this appeal" button is hidden when
//! the logged-in admin DID equals the issuer of the original moderation
//! decision (per `ck.moderation.appeal.review` reducer rule
//! "reviewer.did != original_decision.issuer_did").
//!
//! Verdict picker (uphold / overturn / modify) writes
//! `ck.moderation.appeal.decision` and — when verdict==overturn — the
//! reducer pairs it with `ck.moderation.decision.lift` automatically in
//! the same Seal batch; this UI just surfaces the auto-pairing in a
//! callout so the admin knows what they are about to submit.
//!
//! Wire to `/_soland/admin/moderation/appeals`; sodmin fails closed on
//! separation-of-duties when soland omits the original issuer DID.

use dioxus::prelude::*;

use crate::api::moderation;
use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::components::ui::button::{Button, ButtonSize, ButtonVariant};
use crate::components::ui::card::*;
use crate::components::ui::dialog::ConfirmDialog;
use crate::components::ui::empty_state::EmptyState;
use crate::components::ui::page_header::PageHeader;
use crate::components::ui::table::*;
use crate::components::ui::toast::{ToastVariant, show_toast};
use crate::utils::i18n::t;
use crate::utils::net::session;

/// Wall-clock window after which an appeal auto-closes
/// (`ck.moderation.appeal.close.auto_closed=true`). Spec: 30 days.
const APPEAL_AUTO_CLOSE_DAYS: i64 = 30;

/// Lifecycle state of an appeal as projected from the four
/// `ck.moderation.appeal.*` event variants.
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
    /// i18n key for the lifecycle badge display label.
    fn label_key(&self) -> &'static str {
        match self {
            AppealLifecycle::Submitted => "appeals.state_submitted",
            AppealLifecycle::UnderReview => "appeals.state_under_review",
            AppealLifecycle::Decided => "appeals.state_decided",
            AppealLifecycle::Closed => "appeals.state_closed",
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

    /// Pending appeals are still `submitted` or `under_review`.
    fn is_pending(&self) -> bool {
        matches!(
            self,
            AppealLifecycle::Submitted | AppealLifecycle::UnderReview
        )
    }
}

/// Verdict the reviewing admin can record. Mirrors the SDK
/// `AppealVerdict`. Modify requires the
/// admin to also submit a fresh `ck.moderation.decision` event — that
/// flow is intentionally not wired yet, so the picker greys Modify out
/// with a tooltip.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Verdict {
    Uphold,
    Overturn,
}

impl Verdict {
    /// i18n key for the verdict display label.
    fn label_key(&self) -> &'static str {
        match self {
            Verdict::Uphold => "appeals.verdict_uphold",
            Verdict::Overturn => "appeals.verdict_overturn",
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
/// `ck.schema.moderation_appeal.v1` reducer projection.
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

/// Project a `moderation::AppealRowDto` (one record per
/// appeal_id, latest event) into the page-local [`AppealRow`] view
/// model. Lifecycle is inferred from `appeal_state`; the per-appeal
/// `history` for reviewer notes / decision entries is fetched lazily
/// when the row is selected.
fn project_appeal_row(dto: moderation::AppealRowDto) -> AppealRow {
    let lifecycle = match dto.appeal_state.as_deref() {
        Some("under_review") => AppealLifecycle::UnderReview,
        Some("decided") => AppealLifecycle::Decided,
        Some("closed") => AppealLifecycle::Closed,
        _ => AppealLifecycle::Submitted,
    };
    AppealRow {
        appeal_id: dto.appeal_id,
        decision_ref: dto.decision_ref.unwrap_or_default(),
        target_ref: dto.target_ref.unwrap_or_default(),
        original_issuer_did: dto.original_issuer_did.unwrap_or_default(),
        appellant_did: dto.appellant.unwrap_or_default(),
        reason_text_ref: dto.reason_text_ref.unwrap_or_default(),
        evidence_refs: dto.evidence_refs,
        submitted_at: dto.created_at.unwrap_or_default(),
        // The 30-day countdown is server-derived. soland does not yet
        // expose it, so use the `UNKNOWN` sentinel (rendered as "-") rather
        // than 0 — 0 would paint every fresh appeal as "auto-close imminent"
        // and bury the genuinely near-deadline rows.
        days_until_auto_close: AUTO_CLOSE_UNKNOWN,
        lifecycle,
        reviews: Vec::new(),
        decisions: Vec::new(),
    }
}

#[component]
pub fn ModerationAppealsPage() -> Element {
    let mut selected = use_signal::<Option<String>>(|| None);
    let mut verdict_picker = use_signal::<Option<Verdict>>(|| None);
    let mut confirm_open = use_signal(|| false);
    let mut rows_state = use_signal::<Vec<AppealRow>>(Vec::new);
    let mut load_error = use_signal::<Option<String>>(|| None);
    let mut reload_token = use_signal::<u64>(|| 0);

    // Fetch the live appeals list whenever the page mounts or reload
    // is triggered. The `reload_token` signal is bumped after every
    // successful review/decision/close action so the table refreshes.
    use_effect(move || {
        let token = reload_token.read().to_owned();
        let _ = token; // explicit read so the effect re-runs on bump
        spawn(async move {
            match moderation::list_appeals().await {
                Ok(items) => {
                    let projected: Vec<AppealRow> =
                        items.into_iter().map(project_appeal_row).collect();
                    rows_state.set(projected);
                    load_error.set(None);
                }
                Err(err) => {
                    load_error.set(Some(format!("{}: {err}", t("appeals.load_failed"))));
                }
            }
        });
    });

    let rows = rows_state.read().clone();
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
                title: t("appeals.title"),
                description: format!(
                    "{} {APPEAL_AUTO_CLOSE_DAYS} {}",
                    t("appeals.subtitle_prefix"),
                    t("appeals.subtitle_suffix"),
                ),
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| selected.set(None),
                    {t("common.clear_selection")}
                }
            }

            if pending_only.is_empty() {
                EmptyState {
                    icon_name: "flag".to_string(),
                    title: t("appeals.empty_title"),
                    description: t("appeals.empty_subtitle"),
                }
            } else {
                div { class: "rounded-md border",
                    Table {
                        TableHeader {
                            TableRow {
                                TableHead { {t("appeals.col_appeal")} }
                                TableHead { {t("appeals.col_decision_ref")} }
                                TableHead { {t("appeals.col_appellant")} }
                                TableHead { {t("appeals.col_state")} }
                                TableHead { {t("appeals.col_auto_close")} }
                                TableHead { class: "text-right".to_string(), {t("common.open")} }
                            }
                        }
                        TableBody {
                            for row in pending_only.iter() {
                                {
                                    let row = (*row).clone();
                                    let row_id = row.appeal_id.clone();
                                    let countdown_label = countdown_chip_label(row.days_until_auto_close);
                                    let countdown_variant = countdown_chip_variant(row.days_until_auto_close);
                                    let lifecycle_label = t(row.lifecycle.label_key());
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
                                                    {t("common.open")}
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

            if let Some(ref row) = selected_row {
                {appeal_detail_card(
                    row,
                    &current_admin_did,
                    verdict_picker,
                    confirm_open,
                )}
            }

            {
                let pending = *verdict_picker.read();
                let open = *confirm_open.read();
                let verdict_label = pending.map(|v| t(v.label_key())).unwrap_or_default();
                let description = match pending {
                    Some(Verdict::Overturn) => t("appeals.confirm_overturn_body"),
                    _ => t("appeals.confirm_decision_body"),
                };
                rsx! {
                    ConfirmDialog {
                        open,
                        title: format!("{}: {verdict_label}", t("appeals.record_verdict")),
                        description,
                        confirm_text: format!("{} {verdict_label}", t("appeals.submit")),
                        cancel_text: t("common.cancel"),
                        destructive: matches!(pending, Some(Verdict::Overturn)),
                        on_cancel: move |_| {
                            confirm_open.set(false);
                            verdict_picker.set(None);
                        },
                        on_confirm: {
                            let row = selected_row.clone();
                            let pending_v = pending;
                            move |_| {
                                confirm_open.set(false);
                                verdict_picker.set(None);
                                let Some(row) = row.clone() else { return; };
                                let Some(verdict) = pending_v else { return; };
                                let appeal_id = row.appeal_id.clone();
                                let original_decision_ref = row.decision_ref.clone();
                                let reason_text_ref = row.reason_text_ref.clone();
                                spawn(async move {
                                    // Overturn requires a paired decision-lift; sodmin
                                    // mints the lift via the dedicated admin endpoint
                                    // and threads its identifier back to the appeal
                                    // decision so the reducer's paired-batch check
                                    // in `appeal_decision_overturn_paired_check`
                                    // resolves cleanly.
                                    let mut decision_lift_ref: Option<String> = None;
                                    if matches!(verdict, Verdict::Overturn)
                                        && !original_decision_ref.is_empty()
                                    {
                                        match moderation::lift_decision(
                                            &original_decision_ref,
                                            &moderation::LiftDecisionRequest {
                                                reason_text_ref: Some(reason_text_ref.clone()),
                                                appeal_ref: Some(appeal_id.clone()),
                                            },
                                        )
                                        .await
                                        {
                                            Ok(lift_event) => {
                                                decision_lift_ref = lift_event
                                                    .get("decision_id")
                                                    .and_then(|v| v.as_str())
                                                    .map(ToOwned::to_owned);
                                            }
                                            Err(err) => {
                                                show_toast(
                                                    &format!(
                                                        "{}: {err}", t("appeals.toast_lift_failed")
                                                    ),
                                                    ToastVariant::Error,
                                                );
                                                return;
                                            }
                                        }
                                    }
                                    let verdict_str = match verdict {
                                        Verdict::Uphold => "uphold",
                                        Verdict::Overturn => "overturn",
                                    };
                                    let body = moderation::DecideAppealRequest {
                                        verdict: verdict_str.to_owned(),
                                        reason_text_ref,
                                        modify_decision_ref: None,
                                        decision_lift_ref,
                                    };
                                    match moderation::decide_appeal(&appeal_id, &body).await {
                                        Ok(_) => {
                                            show_toast(
                                                &format!(
                                                    "{} {} (verdict={verdict_str}).",
                                                    t("appeals.toast_recorded_prefix"),
                                                    appeal_id,
                                                ),
                                                ToastVariant::Success,
                                            );
                                            let next = *reload_token.read() + 1;
                                            reload_token.set(next);
                                        }
                                        Err(err) => {
                                            show_toast(
                                                &format!("{}: {err}", t("appeals.toast_decision_failed")),
                                                ToastVariant::Error,
                                            );
                                        }
                                    }
                                });
                            }
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
    //
    // This must fail CLOSED on BOTH unknown sides: when the original issuer
    // DID is missing (handled below via `issuer_known`) AND when the current
    // admin's own DID is missing (viewer fetch failed / localStorage cleared).
    // A missing self-identity previously left `admin_is_issuer = false`,
    // which would let an issuer review their own appeal — fail open.
    let admin_identity_known = !current_admin_did.trim().is_empty();
    let admin_is_issuer = admin_identity_known
        && row
            .original_issuer_did
            .eq_ignore_ascii_case(current_admin_did);
    let issuer_known = !row.original_issuer_did.trim().is_empty();

    let evidence_block: Element = if row.evidence_refs.is_empty() {
        rsx! {
            p { class: "text-sm text-muted-foreground", {t("appeals.no_evidence")} }
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
            p { class: "text-sm text-muted-foreground", {t("appeals.no_reviewer")} }
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
            p { class: "text-sm text-muted-foreground", {t("appeals.no_decision")} }
        }
    } else {
        rsx! {
            ul { class: "space-y-2",
                for entry in row.decisions.iter() {
                    {
                        let label = t(entry.verdict.label_key());
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
                        CardTitle { class: "text-lg".to_string(), {t("appeals.detail_title")} }
                        CardDescription { "{row.appeal_id}" }
                    }
                    Badge { variant: countdown_variant, "{countdown_label}" }
                }
            }
            CardContent { class: "space-y-4".to_string(),
                div { class: "grid gap-3 sm:grid-cols-2",
                    div {
                        p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", {t("appeals.original_decision")} }
                        p { class: "text-xs font-mono break-all", "{row.decision_ref}" }
                    }
                    div {
                        p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", {t("appeals.target")} }
                        p { class: "text-xs font-mono break-all", "{row.target_ref}" }
                    }
                    div {
                        p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", {t("appeals.original_issuer")} }
                        p { class: "text-xs font-mono break-all", "{row.original_issuer_did}" }
                    }
                    div {
                        p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", {t("appeals.appellant")} }
                        p { class: "text-xs font-mono break-all", "{row.appellant_did}" }
                    }
                    div {
                        p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", {t("appeals.submitted_at")} }
                        p { class: "text-xs font-mono", "{row.submitted_at}" }
                    }
                    div {
                        p { class: "text-[10px] uppercase tracking-wider text-muted-foreground", "reason_text_ref" }
                        p { class: "text-xs font-mono break-all", "{row.reason_text_ref}" }
                    }
                }

                div {
                    h3 { class: "text-sm font-semibold mb-1", {t("appeals.evidence")} }
                    {evidence_block}
                }
                div {
                    h3 { class: "text-sm font-semibold mb-1", {t("appeals.reviewer_trail")} }
                    {reviews_block}
                }
                div {
                    h3 { class: "text-sm font-semibold mb-1", {t("appeals.decision_history")} }
                    {decisions_block}
                }

                div { class: "border-t pt-3 space-y-2",
                    h3 { class: "text-sm font-semibold", {t("appeals.review_this")} }
                    if !admin_identity_known {
                        p { class: "rounded-md border border-amber-600/40 bg-amber-600/10 p-2 text-xs text-amber-700 dark:text-amber-300",
                            {t("appeals.review_disabled_self_unknown")}
                        }
                    } else if !issuer_known {
                        p { class: "rounded-md border border-amber-600/40 bg-amber-600/10 p-2 text-xs text-amber-700 dark:text-amber-300",
                            {t("appeals.review_disabled_issuer_unknown")}
                        }
                    } else if admin_is_issuer {
                        // Separation-of-duties — hide the picker entirely.
                        p { class: "rounded-md border border-amber-600/40 bg-amber-600/10 p-2 text-xs text-amber-700 dark:text-amber-300",
                            {t("appeals.sod_self_issuer_prefix")} " (",
                            span { class: "font-mono", "{row.original_issuer_did}" },
                            {t("appeals.sod_self_issuer_suffix")}
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
                                {t("appeals.verdict_uphold")}
                            }
                            Button {
                                variant: ButtonVariant::Destructive,
                                size: ButtonSize::Sm,
                                onclick: move |_| {
                                    verdict_picker.set(Some(Verdict::Overturn));
                                    confirm_open.set(true);
                                },
                                {t("appeals.overturn_auto_pairs")}
                            }
                            Button {
                                variant: ButtonVariant::Outline,
                                size: ButtonSize::Sm,
                                disabled: true,
                                {t("appeals.modify_pending")}
                            }
                        }
                        p { class: "text-xs text-muted-foreground",
                            {t("appeals.overturn_hint")}
                        }
                    }
                }
            }
        }
    }
}

/// Human-readable label for the 30-day auto-close countdown chip in
/// the list view.
/// Sentinel meaning "soland has not exposed the auto-close countdown for
/// this appeal yet". Rendered as "-" / neutral rather than a fake urgency.
pub(crate) const AUTO_CLOSE_UNKNOWN: i64 = i64::MIN;

fn countdown_chip_label(days: i64) -> String {
    if days == AUTO_CLOSE_UNKNOWN {
        "-".to_string()
    } else if days <= 0 {
        "auto-close imminent".to_string()
    } else if days == 1 {
        "1 day to auto-close".to_string()
    } else {
        format!("{days} days to auto-close")
    }
}

/// Red when the appeal is within 5 days of auto-close (the reducer
/// will close it without admin intervention), otherwise neutral. The
/// unknown sentinel is neutral (no countdown data ⇒ no urgency claim).
fn countdown_chip_variant(days: i64) -> BadgeVariant {
    if days == AUTO_CLOSE_UNKNOWN {
        BadgeVariant::Secondary
    } else if days <= 5 {
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
