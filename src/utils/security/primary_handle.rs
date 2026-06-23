//! R3.2 (cokret-spec @ b56cab1, UI-SOD-3) — §3.2.1 primary handle
//! selection, admin-SPA surface.
//!
//! `MemberIdentity` no longer carries `primary_handle` / `handles[]`;
//! handle lifecycle has moved entirely onto signed
//! `ck.schema.handle_claim.v1` evidence. Any admin view that wants to
//! show "the" handle for a subject MUST run the deterministic §3.2.1
//! selection over the visible claim set rather than reading a roster
//! field.
//!
//! This module no longer carries its own copy of the algorithm: the
//! authoritative, wasm-safe implementation now lives in `cokret-core`
//! (`cokret_core::identity::primary_handle`, SOD-05-001 / SPEC-CR-019),
//! and `cokret_core::models::{HandleClaim, HandleBindingState, Handle}`
//! are the exact types the admin DTOs already re-export. We simply
//! re-export the core helpers so yougen / soland / cotest / sodmin all
//! agree on which claim wins from a single source of truth.
//!
//! The selection is a pure function of an explicit six-tuple:
//! `(subject_id, context, claim_set_snapshot, accepted_issuers,
//! holder_primary_handle_at_as_of, resolution_as_of)`.
//!
//! Step 0 — candidate pre-filter: `binding_state == Verified`,
//! `created_at <= resolution_as_of`, `expires_at > resolution_as_of`,
//! issuer ∈ `accepted_issuers`, audience scope match.
//! Step 1 — priority layers: audience-matched > holder-flagged >
//! most-recent.
//! Step 2 — deterministic tie-breaker: `accepted_issuers` position
//! (earlier wins) → `created_at` (later wins) → canonical handle
//! string (lexicographically smaller wins).

pub use cokret_core::identity::primary_handle::{
    PrimaryHandleSelectInput, SubjectRender, render_subject, select_primary_handle_string,
};
