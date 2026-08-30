//! Admin pages for Realm security boundaries.

/// Realm link-graph read-only operations view.
pub mod links;
pub mod list;
/// Realm `media_service.foci[]` read-only view.
pub mod media_service;
pub mod multisig;
pub mod notary;
/// Realm organization relationships and principal-control projections.
pub mod organization;
pub mod seal_dag;
pub mod show;
