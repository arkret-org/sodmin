//! Admin pages for Realm security boundaries.

/// Realm delivery-binding-policy read-only operations view.
pub mod delivery_binding;
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
