//! Admin pages for Realm security boundaries.

/// Current governing Station, Realm commit stream head and handoff chain.
pub mod authority;

/// Realm link-graph read-only operations view.
pub mod links;
pub mod list;
/// Realm `media_service.foci[]` read-only view.
pub mod media_service;
/// Realm organization relationships and principal-control projections.
pub mod organization;
pub mod show;
