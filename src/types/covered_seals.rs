//! DTO shapes for the E2EE covered_seals lag admin surface (Stream H',
//! H'7).
//!
//! Read from `GET /_soland/admin/realms/{id}/mls/covered-seals` —
//! soland projects the covered-seals state for the
//! `ck:cell:ck.component.covered_seals.v1:<realm_id>` cell. The cell
//! records which governance Seals the MLS group has acknowledged. We compare
//! the current governance Seal set against the covered set to compute a lag
//! count.
//!
//! The wire DTOs, the lag helpers, and the warn threshold constant now live
//! in `soland-core` so this consumer and the soland producer share a single
//! definition (mirrors the `soland_core::admin::seal` sharing pattern). This
//! module re-exports them so downstream reference points stay unchanged.

pub use soland_core::admin::covered_seals::{
    CoveredSealsAdvanceOutcome, CoveredSealsSnapshot, DEFAULT_LAG_WARN_THRESHOLD,
};
