//! Stop-gap "unwired field" annotations (review D14).
//!
//! Several admin surfaces are backed by soland's **dev-only** snapshot
//! endpoints (`/_soland/admin/actors`, `/_soland/admin/audit`). Those endpoints
//! never emit a batch of fields, so the wire value falls back to the Rust
//! default (`false` / `0` / `None`). Rendering that default as a definite value
//! ("No", "0", "-") silently misleads operators — a suspended or admin
//! principal would read as a confident negative.
//!
//! These helpers render an explicit "not wired / data unavailable" marker
//! instead. Security-critical fields (admin / suspended / deactivated flags) use
//! the `critical` variant so they are visually distinct and never mistaken for a
//! trustworthy `false`.
//!
//! This is deliberately UI-only triage. The real fix — migrating these surfaces
//! off the dev snapshot onto production projections — is tracked in the backlog
//! and is intentionally NOT attempted here.

use dioxus::prelude::*;

use crate::utils::i18n::t;

/// Inline "unavailable" badge for a single unwired field value.
///
/// `critical == true` selects the amber security-flag styling; use it for the
/// admin / suspended / deactivated flags where a false negative is dangerous.
/// `critical == false` is the neutral variant for non-security fields (activity
/// timestamps, counters).
pub fn unwired_badge(critical: bool) -> Element {
    let (class, testid, label_key) = if critical {
        (
            "inline-flex items-center gap-1 rounded-full border border-amber-600/60 bg-amber-600/10 px-2 py-0.5 text-xs font-medium text-amber-700 dark:text-amber-300",
            "unwired-field-critical",
            "unwired.badge_critical",
        )
    } else {
        (
            "inline-flex items-center gap-1 rounded-full border border-border bg-muted/40 px-2 py-0.5 text-xs font-medium text-muted-foreground",
            "unwired-field",
            "unwired.badge",
        )
    };

    rsx! {
        span {
            class,
            "data-testid": testid,
            title: t("unwired.tooltip"),
            "aria-label": t("unwired.tooltip"),
            // Em dash keeps the marker legible even if the label wraps away.
            span { class: "leading-none", "\u{2014}" }
            {t(label_key)}
        }
    }
}

/// A label / value overview row whose value is unwired. Matches the two-column
/// `field_row` layout used by the actor overview card so unwired and wired rows
/// line up.
pub fn unwired_field_row(label: String, critical: bool) -> Element {
    rsx! {
        div { class: "flex justify-between py-1.5 border-b border-border/50 last:border-0",
            span { class: "text-sm text-muted-foreground", "{label}" }
            {unwired_badge(critical)}
        }
    }
}

/// Compact marker appended to a table column header whose entire column is
/// unwired (every cell is a placeholder default). Keeps the per-row cells quiet
/// while documenting, once at the top, that the column carries no real data.
pub fn unwired_header_note() -> Element {
    rsx! {
        span {
            class: "ml-1.5 inline-flex items-center rounded-full border border-amber-600/50 bg-amber-600/10 px-1.5 py-0.5 text-[10px] font-medium uppercase tracking-wide text-amber-700 dark:text-amber-300",
            "data-testid": "unwired-column",
            title: t("unwired.tooltip"),
            "aria-label": t("unwired.tooltip"),
            {t("unwired.column_note")}
        }
    }
}
