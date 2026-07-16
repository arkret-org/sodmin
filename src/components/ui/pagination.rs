//! `Pagination` / `CursorPagination` — migrated to yoface
//! (`yoface::ui::pagination::*`).
//!
//! The yoface versions' signatures match (with new optional `prev_label` /
//! `next_label` props that have defaults), so call sites need zero changes.
//! Internally they reuse yoface's `Button`, with rendering going through
//! css_module + the Soft Orbit tokens.
pub use yoface::ui::pagination::{CursorPagination, Pagination};
