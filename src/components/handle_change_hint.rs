//! R3.2 (UI-SOD-5) — "handle changed since" history hint.
//!
//! Mention / audit / history surfaces capture the handle that was shown
//! *at the time* of the event (`handle_at_time`, audit metadata only).
//! When that captured handle differs from the subject's *current*
//! primary handle (derived via §3.2.1 selection), the UI surfaces a
//! non-blocking hint so an operator reading historical data understands
//! the actor's handle has since changed. This is a display-layer
//! enhancement only — `subject_id` (not the handle string) remains the
//! authoritative attribution field.

use dioxus::prelude::*;

use crate::components::ui::badge::{Badge, BadgeVariant};
use crate::utils::i18n::t;

/// Returns `true` when the captured historical handle is present and
/// differs from the current primary handle (both non-empty). A missing
/// current handle (selection unresolved) is NOT treated as a change —
/// we only flag a confirmed drift.
pub fn handle_changed(handle_at_time: Option<&str>, current_primary: Option<&str>) -> bool {
    match (handle_at_time, current_primary) {
        (Some(was), Some(now)) if !was.is_empty() && !now.is_empty() => was != now,
        _ => false,
    }
}

/// Inline "handle changed since" hint. Renders nothing when the handle
/// has not changed, so callers can drop it unconditionally next to a
/// historical handle label.
#[component]
pub fn HandleChangeHint(
    /// The handle captured at event time (`handle_at_time`).
    handle_at_time: Option<String>,
    /// The subject's current primary handle (from §3.2.1 selection).
    current_primary: Option<String>,
) -> Element {
    if !handle_changed(handle_at_time.as_deref(), current_primary.as_deref()) {
        return rsx! {};
    }
    let now = current_primary.clone().unwrap_or_default();
    let was = handle_at_time.clone().unwrap_or_default();
    let title = format!("{}: {} \u{2192} {}", t("handle_change.now"), was, now);
    rsx! {
        span { title: "{title}",
            Badge {
                variant: BadgeVariant::Outline,
                class: "ml-1 text-[10px] text-amber-700 dark:text-amber-300 border-amber-600/40".to_string(),
                {t("handle_change.changed_since")}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_change_only_when_both_present_and_differ() {
        assert!(handle_changed(Some("a:x"), Some("b:x")));
        assert!(!handle_changed(Some("a:x"), Some("a:x")));
        assert!(!handle_changed(Some("a:x"), None));
        assert!(!handle_changed(None, Some("b:x")));
        assert!(!handle_changed(Some(""), Some("b:x")));
    }
}
