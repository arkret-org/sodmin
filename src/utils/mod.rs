pub mod audit;
pub mod cache;
pub mod config;
pub mod crypto;
pub mod csp;
/// B.7 — client-side CSV export for admin list pages.
pub mod csv;
pub mod date;
/// Round 4 — DID input validation (tightened `^did:[a-z0-9]+:[^\s]+$`).
pub mod did;
pub mod error;
/// R3 — Handle localpart NFC / script-mixed / confusable check that
/// mirrors the SDK helper for inline `handle_homograph_forbidden`
/// warnings.
pub mod handle;
/// Concurrent `join_all` primitive used by bulk admin actions.
pub mod futures;
pub mod i18n;
pub mod password;
pub mod perf;
/// Client-side `filter[name_or_id]` substring matcher used as a
/// fallback when a backend hasn't yet plumbed the parameter.
pub mod search;
pub mod session;
pub mod storage;
/// P5 — opt-in browser-error telemetry and request_id formatting
/// helpers. Requires `SODMIN_TELEMETRY_ENDPOINT` at deploy time.
pub mod telemetry;
