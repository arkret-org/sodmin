//! Checkbox has been migrated to yoface (`yoface::ui::checkbox`).
//!
//! Both the "select all" header and the row selection on sodmin's list pages
//! use the tristate variant, so the local `Checkbox` alias points at yoface's
//! `TristateCheckbox` (whose API is a superset of the local two-state version:
//! one extra `indeterminate`, defaulting to false). The `header_state` pure
//! function is the same one, so it is re-exported directly.
pub use yoface::ui::checkbox::{TristateCheckbox as Checkbox, header_state};
