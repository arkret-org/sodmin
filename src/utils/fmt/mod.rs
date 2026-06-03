/// B.7 — client-side CSV export for admin list pages.
pub mod csv;
pub mod date;
/// Client-side `filter[name_or_id]` substring matcher used as a
/// fallback when a backend hasn't yet plumbed the parameter.
pub mod search;
