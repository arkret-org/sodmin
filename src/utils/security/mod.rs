pub mod crypto;
/// Handle display helpers for admin read-only views.
pub mod handle;
/// §3.2.1 primary handle selection (admin-SPA mirror
/// of the SDK `select_primary_handle` helper). `MemberIdentity` no
/// longer carries handle fields; the UI derives the display handle by
/// running this deterministic selection over the visible claim set.
pub mod primary_handle;
