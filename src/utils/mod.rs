pub mod audit;
pub mod cache;
pub mod config;
pub mod crypto;
pub mod csp;
/// B.7 — client-side CSV export for admin list pages.
pub mod csv;
pub mod date;
/// DID input validation — delegates to the SDK scalar validator
/// `contrix_identifiers::is_did` (global report #10, candidate 12).
pub mod did;
pub mod error;
/// Concurrent `join_all` primitive used by bulk admin actions.
pub mod futures;
/// R3 — Handle localpart NFC / script-mixed / confusable check that
/// mirrors the SDK helper for inline `handle_homograph_forbidden`
/// warnings.
pub mod handle;
pub mod i18n;
pub mod password;
pub mod perf;
/// R3.2 (UI-SOD-3) — §3.2.1 primary handle selection (admin-SPA mirror
/// of the SDK `select_primary_handle` helper). `MemberIdentity` no
/// longer carries handle fields; the UI derives the display handle by
/// running this deterministic selection over the visible claim set.
pub mod primary_handle;
/// Client-side `filter[name_or_id]` substring matcher used as a
/// fallback when a backend hasn't yet plumbed the parameter.
pub mod search;
pub mod session;
pub mod storage;
/// Opt-in browser-error telemetry. Requires `SODMIN_TELEMETRY_ENDPOINT`
/// at deploy time.
pub mod telemetry;
