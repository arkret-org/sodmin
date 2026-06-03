//! R3.2 (cokret-spec @ b56cab1, UI-SOD-3) — §3.2.1 primary handle
//! selection, admin-SPA mirror.
//!
//! `MemberIdentity` no longer carries `primary_handle` / `handles[]`;
//! handle lifecycle has moved entirely onto signed
//! `ck.schema.handle_claim.v1` evidence. Any admin view that wants to
//! show "the" handle for a subject MUST run the deterministic §3.2.1
//! selection over the visible claim set rather than reading a roster
//! field. This module mirrors the SDK helper
//! `cokret::identity::select_primary_handle` so the admin UI agrees
//! with yougen / soland / cotest on which claim wins.
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
//! (earlier wins) → `created_at` (later wins) → `claim_digest`
//! (lexicographically smaller wins; falls back to claim handle string
//! when the server did not pre-compute a digest hint).

use chrono::{DateTime, Utc};

use crate::types::{HandleBindingState, HandleClaim};

/// Deterministic six-tuple input to [`select_primary_handle`]. Mirrors
/// the SDK `PrimaryHandleSelectInput`.
pub struct PrimaryHandleSelectInput<'a> {
    /// Holder/principal DID being resolved. Authoritative attribution
    /// key — never a Realm `actor_id`.
    pub subject_id: &'a str,
    /// Current resolution context (target Realm id / inviting service
    /// DID) matched against `claim.audience`. `None` = no audience
    /// constraint.
    pub context: Option<&'a str>,
    /// Currently visible claim set snapshot (e.g.
    /// `member_roster_entry.handle_claims[]`).
    pub claim_set_snapshot: &'a [HandleClaim],
    /// Realm policy `accepted_issuers` in trust order (earlier == more
    /// trusted). Claims whose issuer is absent are dropped in Step 0.
    pub accepted_issuers: &'a [String],
    /// `metadata.primary_handle` (holder preference) at
    /// `resolution_as_of`. `None` skips the holder-flagged layer.
    /// TODO(R3.2.1): resolve from the subject DID Document snapshot.
    pub holder_primary_handle_at_as_of: Option<&'a str>,
    /// As-of instant the selection runs against.
    pub resolution_as_of: DateTime<Utc>,
}

/// §3.2.1 — select the deterministic primary-handle claim for a subject
/// in a context. Returns `None` when the candidate set is empty (the
/// caller MUST then follow the §3.8.2 unresolved fallback path).
pub fn select_primary_handle(input: &PrimaryHandleSelectInput<'_>) -> Option<HandleClaim> {
    // Step 0 — candidate pre-filter.
    let candidates: Vec<&HandleClaim> = input
        .claim_set_snapshot
        .iter()
        .filter(|c| candidate_passes_step0(c, input))
        .collect();
    if candidates.is_empty() {
        return None;
    }

    // Step 1 — priority layers.
    let layer: Vec<&HandleClaim> = {
        let audience_matched: Vec<&HandleClaim> = candidates
            .iter()
            .copied()
            .filter(|c| matches_audience(c, input.context))
            .collect();
        if !audience_matched.is_empty() {
            audience_matched
        } else {
            let holder_flagged: Vec<&HandleClaim> = candidates
                .iter()
                .copied()
                .filter(|c| holder_flagged(c, input.holder_primary_handle_at_as_of))
                .collect();
            if !holder_flagged.is_empty() {
                holder_flagged
            } else {
                candidates.clone()
            }
        }
    };

    // Step 2 — deterministic tie-breaker.
    let winner = layer.into_iter().reduce(|best, candidate| {
        if tie_break_prefers(candidate, best, input.accepted_issuers) {
            candidate
        } else {
            best
        }
    })?;
    Some(winner.clone())
}

/// Convenience: run [`select_primary_handle`] and return only the
/// canonical handle string of the winning claim (if any). Admin views
/// that just need "the handle to show" use this.
pub fn select_primary_handle_string(input: &PrimaryHandleSelectInput<'_>) -> Option<String> {
    select_primary_handle(input).and_then(|c| c.handle)
}

/// Visual-degradation tier the UI MUST surface when rendering a subject.
/// Mirrors the SDK `MentionRender`. Used by history / mention surfaces
/// (UI-SOD-5) so each fallback step carries a distinct degraded badge.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubjectRender {
    /// Resolved to a verified primary handle via §3.2.1.
    Verified(String),
    /// Served from a captured display name (degraded).
    NameOnly(String),
    /// Nothing resolved; UI shows a truncated DID (degraded).
    Unresolved(String),
}

/// §3.8.2 — render a subject from its authoritative `subject_id`,
/// degrading through display name then truncated DID. `subject_id` is
/// the only authoritative attribution field; `display_name_at_time` is
/// audit metadata used purely for the degraded label.
pub fn render_subject(
    input: &PrimaryHandleSelectInput<'_>,
    display_name_at_time: Option<&str>,
) -> SubjectRender {
    if let Some(handle) = select_primary_handle_string(input) {
        return SubjectRender::Verified(handle);
    }
    if let Some(name) = display_name_at_time {
        return SubjectRender::NameOnly(name.to_owned());
    }
    SubjectRender::Unresolved(truncate_did(input.subject_id))
}

fn truncate_did(did: &str) -> String {
    if did.len() <= 16 {
        return did.to_owned();
    }
    let head = &did[..10.min(did.len())];
    let tail = &did[did.len().saturating_sub(3)..];
    format!("{head}\u{2026}{tail}")
}

fn candidate_passes_step0(c: &HandleClaim, input: &PrimaryHandleSelectInput<'_>) -> bool {
    if !matches!(c.binding_state, Some(HandleBindingState::Verified)) {
        return false;
    }
    // created_at MUST be <= resolution_as_of.
    match parse_ts(c.created_at.as_deref()) {
        Some(created) if created <= input.resolution_as_of => {}
        _ => return false,
    }
    // expires_at MUST be > resolution_as_of.
    match parse_ts(c.expires_at.as_deref()) {
        Some(expiry) if expiry > input.resolution_as_of => {}
        _ => return false,
    }
    // issuer trust filter (mandatory pre-filter).
    match &c.issuer {
        Some(issuer) if input.accepted_issuers.iter().any(|i| i == issuer) => {}
        _ => return false,
    }
    // audience scope filter: present audience must equal context.
    if let Some(aud) = &c.audience {
        match input.context {
            Some(ctx) if ctx == aud => {}
            _ => return false,
        }
    }
    true
}

fn matches_audience(c: &HandleClaim, context: Option<&str>) -> bool {
    matches!((c.audience.as_deref(), context), (Some(a), Some(ctx)) if a == ctx)
}

fn holder_flagged(c: &HandleClaim, holder_primary: Option<&str>) -> bool {
    match (c.handle.as_deref(), holder_primary) {
        (Some(handle), Some(pref)) => handle == pref,
        _ => false,
    }
}

/// Returns `true` if `candidate` should win over `best` per the Step 2
/// tie-breaker ordering.
fn tie_break_prefers(
    candidate: &HandleClaim,
    best: &HandleClaim,
    accepted_issuers: &[String],
) -> bool {
    let cand_pos = issuer_position(candidate, accepted_issuers);
    let best_pos = issuer_position(best, accepted_issuers);
    if cand_pos != best_pos {
        return cand_pos < best_pos;
    }
    let cand_created = parse_ts(candidate.created_at.as_deref());
    let best_created = parse_ts(best.created_at.as_deref());
    if cand_created != best_created {
        return cand_created > best_created;
    }
    // claim_digest tie-break — fall back to the handle string when the
    // server did not pre-compute a digest hint (the SPA does not run the
    // full sha256(JCS(semantic_projection)) — TODO(R3.2.1)).
    let cand_key = candidate
        .claim_digest
        .as_deref()
        .or(candidate.handle.as_deref());
    let best_key = best.claim_digest.as_deref().or(best.handle.as_deref());
    match (cand_key, best_key) {
        (Some(cd), Some(bd)) => cd < bd,
        _ => false,
    }
}

fn issuer_position(c: &HandleClaim, accepted_issuers: &[String]) -> usize {
    match &c.issuer {
        Some(issuer) => accepted_issuers
            .iter()
            .position(|i| i == issuer)
            .unwrap_or(usize::MAX),
        None => usize::MAX,
    }
}

fn parse_ts(s: Option<&str>) -> Option<DateTime<Utc>> {
    s.and_then(|raw| {
        DateTime::parse_from_rfc3339(raw)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verified(
        handle: &str,
        issuer: &str,
        created: &str,
        expires: &str,
        audience: Option<&str>,
    ) -> HandleClaim {
        HandleClaim {
            handle: Some(handle.to_string()),
            subject: Some("did:web:alice.example".to_string()),
            issuer: Some(issuer.to_string()),
            binding_state: Some(HandleBindingState::Verified),
            audience: audience.map(str::to_string),
            created_at: Some(created.to_string()),
            expires_at: Some(expires.to_string()),
            ..Default::default()
        }
    }

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-05-28T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn empty_candidate_set_returns_none() {
        let input = PrimaryHandleSelectInput {
            subject_id: "did:web:alice.example",
            context: None,
            claim_set_snapshot: &[],
            accepted_issuers: &[],
            holder_primary_handle_at_as_of: None,
            resolution_as_of: now(),
        };
        assert!(select_primary_handle(&input).is_none());
    }

    #[test]
    fn audience_match_wins_over_most_recent() {
        let acc = vec![
            "did:web:acme.example".to_string(),
            "did:web:other.example".to_string(),
        ];
        let matching = verified(
            "alice:acme.example",
            "did:web:acme.example",
            "2026-05-28T08:00:00Z",
            "2026-06-28T00:00:00Z",
            Some("ck:realm:r1"),
        );
        let newer = verified(
            "alice:other.example",
            "did:web:other.example",
            "2026-05-28T11:00:00Z",
            "2026-06-28T00:00:00Z",
            None,
        );
        let snapshot = vec![newer, matching];
        let input = PrimaryHandleSelectInput {
            subject_id: "did:web:alice.example",
            context: Some("ck:realm:r1"),
            claim_set_snapshot: &snapshot,
            accepted_issuers: &acc,
            holder_primary_handle_at_as_of: None,
            resolution_as_of: now(),
        };
        let chosen = select_primary_handle(&input).unwrap();
        assert_eq!(chosen.handle.as_deref(), Some("alice:acme.example"));
    }

    #[test]
    fn issuer_not_in_accepted_is_dropped() {
        let claim = verified(
            "alice:rogue.example",
            "did:web:rogue.example",
            "2026-05-28T08:00:00Z",
            "2026-06-28T00:00:00Z",
            None,
        );
        let snapshot = vec![claim];
        let input = PrimaryHandleSelectInput {
            subject_id: "did:web:alice.example",
            context: None,
            claim_set_snapshot: &snapshot,
            accepted_issuers: &["did:web:acme.example".to_string()],
            holder_primary_handle_at_as_of: None,
            resolution_as_of: now(),
        };
        assert!(select_primary_handle(&input).is_none());
    }

    #[test]
    fn expired_claim_is_dropped() {
        let claim = verified(
            "alice:acme.example",
            "did:web:acme.example",
            "2026-05-01T08:00:00Z",
            "2026-05-02T00:00:00Z",
            None,
        );
        let snapshot = vec![claim];
        let input = PrimaryHandleSelectInput {
            subject_id: "did:web:alice.example",
            context: None,
            claim_set_snapshot: &snapshot,
            accepted_issuers: &["did:web:acme.example".to_string()],
            holder_primary_handle_at_as_of: None,
            resolution_as_of: now(),
        };
        assert!(select_primary_handle(&input).is_none());
    }
}
