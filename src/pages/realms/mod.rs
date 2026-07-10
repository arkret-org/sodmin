//! Admin pages for Realm security boundaries.

pub mod covered_seals;
/// Realm delivery-binding-policy read-only operations view.
pub mod delivery_binding;
/// Round R2/R3 — Realm destroy page (T07).
pub mod destroy;
pub mod federation_status;
/// R3.1 (MID-3) — Realm identity audit diagnostic page.
pub mod identity_audit;
pub mod links;
pub mod list;
/// R3 (UI-3) — Realm `media_service.foci[]` read-only view.
pub mod media_service;
pub mod multisig;
pub mod notary;
/// Realm organization relationships and principal-control projections.
pub mod organization;
pub mod seal_dag;
pub mod show;
pub mod signing_keys;
