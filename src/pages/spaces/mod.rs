//! Admin pages for "Space" rows.
//!
//! Realm-rework note: most modules here address the **security boundary**
//! (which the spec now calls a Realm — `cx:realm:` URI prefix). Pure
//! container Spaces (`cx:space:` URI prefix) are not surfaced separately
//! yet. Modules that touch the security boundary are flagged below; the
//! rest will be revisited once the directory exposes container-Space
//! rows distinctly.
// TODO(realm-rework): split this module into `realms` (security
// boundary: policy_editor, signing_keys, anchorer, multisig,
// admin_list, federation_status, covered_frontier, consent,
// signing_keys) and `spaces` (container: hierarchy, list, create,
// bottom, components, show, anchor_dag) once the API splits.
pub mod admin_list;
pub mod anchor_dag;
pub mod anchorer;
pub mod bottom;
pub mod components;
pub mod consent;
pub mod covered_frontier;
pub mod create;
pub mod federation_status;
pub mod hierarchy;
pub mod list;
pub mod multisig;
pub mod policy_editor;
pub mod show;
pub mod signing_keys;
