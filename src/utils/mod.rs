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
pub mod i18n;
pub mod password;
pub mod perf;
pub mod session;
pub mod storage;
