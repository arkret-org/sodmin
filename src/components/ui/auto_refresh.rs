//! Shared "Auto-refresh" interval picker for list pages.
//!
//! Renders a small `<select>` with the four canonical intervals (off,
//! 30s, 1m, 5m) and persists the choice to LocalStorage so an admin's
//! cadence sticks across reloads. The caller wires the chosen interval
//! into a [`gloo_timers::callback::Interval`] (or
//! `gloo_timers::future::IntervalStream`) and re-fetches when it
//! fires.
//!
//! Each list page passes a unique `storage_key` so cadence preferences
//! don't bleed between pages (an admin watching `/devices` at 30s
//! shouldn't churn `/federation` at the same rate).

use dioxus::prelude::*;

use crate::utils::storage;

/// Discrete interval choices in milliseconds. The `Off` variant means
/// no timer is installed — the parent should skip the
/// `Interval::new` call entirely. The four-step ladder is deliberate:
/// less granularity means fewer ways to wire the timer wrong, and
/// covers "watch live" / "monitor a job" / "check periodically" /
/// "leave it open in a tab".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshInterval {
    Off,
    ThirtySec,
    OneMin,
    FiveMin,
}

impl RefreshInterval {
    pub const ALL: [RefreshInterval; 4] = [
        RefreshInterval::Off,
        RefreshInterval::ThirtySec,
        RefreshInterval::OneMin,
        RefreshInterval::FiveMin,
    ];

    /// Wire value persisted to LocalStorage. Stable strings so a
    /// browser session survives a code rename of the enum variant.
    pub fn wire(self) -> &'static str {
        match self {
            RefreshInterval::Off => "off",
            RefreshInterval::ThirtySec => "30s",
            RefreshInterval::OneMin => "1m",
            RefreshInterval::FiveMin => "5m",
        }
    }

    pub fn from_wire(s: &str) -> Self {
        match s {
            "30s" => RefreshInterval::ThirtySec,
            "1m" => RefreshInterval::OneMin,
            "5m" => RefreshInterval::FiveMin,
            _ => RefreshInterval::Off,
        }
    }

    /// Display label for the `<option>`. `None` when off so the picker
    /// can show a localized "Off" string supplied by the caller.
    pub fn label(self) -> &'static str {
        match self {
            RefreshInterval::Off => "Off",
            RefreshInterval::ThirtySec => "30s",
            RefreshInterval::OneMin => "1m",
            RefreshInterval::FiveMin => "5m",
        }
    }

    /// Interval in milliseconds, or `None` for `Off`. Callers use
    /// `None` to skip installing a timer.
    pub fn millis(self) -> Option<u32> {
        match self {
            RefreshInterval::Off => None,
            RefreshInterval::ThirtySec => Some(30_000),
            RefreshInterval::OneMin => Some(60_000),
            RefreshInterval::FiveMin => Some(300_000),
        }
    }
}

/// Read the persisted interval for `storage_key`, defaulting to
/// `Off` when nothing is stored (admins opt in explicitly).
pub fn load(storage_key: &str) -> RefreshInterval {
    storage::get_item(storage_key)
        .map(|raw| RefreshInterval::from_wire(&raw))
        .unwrap_or(RefreshInterval::Off)
}

/// Persist the chosen interval. `Off` writes the literal `"off"`
/// string so a future load returns the same value rather than the
/// default — important if the page-level default ever changes.
pub fn save(storage_key: &str, value: RefreshInterval) {
    storage::set_item(storage_key, value.wire());
}

#[component]
pub fn AutoRefreshPicker(
    /// LocalStorage key — each list page picks a distinct one.
    storage_key: String,
    value: RefreshInterval,
    onchange: EventHandler<RefreshInterval>,
) -> Element {
    let key_for_change = storage_key.clone();
    rsx! {
        label { class: "flex items-center gap-2 text-sm text-muted-foreground",
            span { "Auto-refresh" }
            select {
                class: "h-9 rounded-md border border-input bg-background px-2 text-sm focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring",
                aria_label: "Auto-refresh interval",
                value: value.wire(),
                onchange: move |evt| {
                    let next = RefreshInterval::from_wire(&evt.value());
                    save(&key_for_change, next);
                    onchange.call(next);
                },
                for option in RefreshInterval::ALL.iter() {
                    option {
                        value: option.wire(),
                        selected: *option == value,
                        "{option.label()}"
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_roundtrip() {
        for opt in RefreshInterval::ALL.iter() {
            assert_eq!(RefreshInterval::from_wire(opt.wire()), *opt);
        }
    }

    #[test]
    fn unknown_wire_falls_back_to_off() {
        assert_eq!(RefreshInterval::from_wire("nope"), RefreshInterval::Off);
        assert_eq!(RefreshInterval::from_wire(""), RefreshInterval::Off);
    }

    #[test]
    fn millis_off_returns_none() {
        assert_eq!(RefreshInterval::Off.millis(), None);
        assert_eq!(RefreshInterval::ThirtySec.millis(), Some(30_000));
        assert_eq!(RefreshInterval::FiveMin.millis(), Some(300_000));
    }
}
