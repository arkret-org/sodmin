pub mod crypto;
pub mod csp;
/// DID input validation — delegates to the SDK scalar validator
/// `cokret_identifiers::is_did` (global report #10, candidate 12).
pub mod did;
/// R3 — Handle localpart NFC / script-mixed / confusable check that
/// mirrors the SDK helper for inline `handle_homograph_forbidden`
/// warnings.
pub mod handle;
pub mod password;
/// R3.2 (UI-SOD-3) — §3.2.1 primary handle selection (admin-SPA mirror
/// of the SDK `select_primary_handle` helper). `MemberIdentity` no
/// longer carries handle fields; the UI derives the display handle by
/// running this deterministic selection over the visible claim set.
pub mod primary_handle;
