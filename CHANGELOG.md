# Changelog

All notable changes to `sodmin` (Contrix Server admin UI) are documented in
this file. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this project is pre-1.0 so wire-breaking changes can land in any release until
the SDK pins down its 1.0 contract.

## R3.3 — Spec sync 2026-05-28 (contrix-spec @ cced4b8)

- R3.3 spec sync — pin to contrix-spec @ cced4b8 (CXP-0011). `resolve_target` operator diagnostic page deferred to R3.3.1.

> No version tag, no crates.io / Docker Hub / npm publish — git commit only.
## R3.2 — Spec sync 2026-05-28 (contrix-spec @ b56cab1)

- Renamed roster `identity_state_digest` → `member_display_state_digest`; added roster v2 gated fields + `HandleClaim`/`HandleBindingState` mirrors (`claim_type` drops `service_handle`).
- New `utils/primary_handle.rs` mirrors SDK §3.2.1 selection; actor/identity views derive the handle via selection (MemberIdentity handle fields removed).
- New `/admin/handles/by-subject` page calling `list_handles_for_subject` with a "Why am I seeing this?" tooltip; "handle changed since" hint on the identity-audit page.
- Claim-set join + accepted_issuers policy + DID-doc holder preference deferred `TODO(R3.2.1)`.

> No version tag, no crates.io / Docker Hub / npm publish — git commit only.
## R3 — Spec sync 2026-05-27 (contrix-spec @ b47ff6ec)

- UI-1 / UI-2: Agent list status badges (`active` / `paused` / `deactivated`) with pause / resume / deactivate / rotate-key / grants actions wired to `/agents/{id}/deactivate`; draft / action_request / approve / reject lifecycle stubbed.
- UI-3: Realm settings page `/realms/:realm_id/media-service` for editing `media_service.foci[]` (livekit / mediasoup / janus / contrix-native / moq-relay) with a migration banner for legacy `sfu_endpoint`.
- UI-4 / UI-5: Recovery policy stub view at `/coauth/recovery` listing receipts with `proof_summary[]`; handle homograph inline NFC + script-mixed warning on `/actors/create`.
- UI-6: Server profile chip surface at `/server-status` showing declared / absent state for `accountable_to.strict_reject.v1`, `media_service_binding.v1` (+ livekit / contrix_native), and `key_backup.memory_hard.v1`.
- UI-7: localized en + zh-CN strings for the new errcodes (`pairing_request_expired`, `proof_invalid`, `agent_paused`, `agent_deactivated`, `accountability_grant_missing`, `handle_homograph_forbidden`, `participant_binding_invalid`, `legacy_single_endpoint_media_service`) rendered via `ErrorBanner`.

> No version tag, no crates.io / Docker Hub / npm publish — git commit only.

## [Unreleased]

### CXP-0007 Circle rollout (P3A)

UI surfaces for the encrypted-sub-boundary primitive shipped by
soland P2A + coauth P2B.

- **Added** Circle management UI (P3A.3): `/circles`, `/circles/new`,
  `/circles/:id`, `/circles/:id/members`, `/circles/:id/scope` Dioxus
  pages plus a `src/api/circles.rs` fetch wrapper. Eight `/api/v1/circles/*`
  routes added to `build.rs` REQUIRED_SOLAND so a missing route fails
  the wasm build at contract-check time.
- **Added** Capability grant dialog (P3A.4) gains a quick-select for the
  six `cx.circle.*` actions and an `allowed_circle_refs` CSV input
  that emits the GrantConstraint server-side.
- **Added** Audit log (P3A.5) renders the new `effective_scope` field
  with a deep link into `/circles/:id` for Circle-scoped events, and
  the filter row grows an event-kind dropdown covering the seven
  `cx.circle.*` event kinds.
- **Added** Realm classification badges (P3A.6): new
  `RealmClassificationBadge` component renders the Principal
  Control / Collaboration / unknown distinction. The Realm/Space
  create form now requires the immutable classification at create
  time (defaults to `collaboration`).
- **Added** CXP-0007 reason-code i18n (P3A.8): six reducer
  rejection reasons (`circle_realm_mismatch`,
  `circle_member_must_be_realm_member`, `circle_not_active`,
  `circle_already_terminal`, `circle_capability_denied`,
  `circle_scope_rotation_in_progress`) now render with localised
  toast bodies.
- **Added** Playwright e2e `tests/e2e/circle-admin.spec.ts` covering
  create / valid member / subset-rejection / archive / audit fanout
  and a capability grant happy path.
- **Fixed** `.gitignore` tightening (P3A.1) — explicit pins for
  `/bundle.js.map`, `/docker-build.log`, and `**/dist/` so root-level
  build artifacts cannot regress past the wildcard rules.
- **Notes** Version unchanged; this round ships against the existing
  `circle-rollout` branch only. Lighthouse perf budget raised from
  70 → 80 to track industry baseline.

### Round R4 — protocol review closures (2026-05-20, contrix-spec `2a4d39b..a77b995`)

UI surfaces for the round-4 wire changes implemented in soland / coauth.
See [`../_todos.md`](../_todos.md) for the workstream context.

- **Added** `ServiceDescribe` v2 admin view — all 17 required fields
  rendered; `development_mode=true` paired with non-empty
  `verified_profiles` raises a red warning row.
- **Added** `/server/trust-domain` config page now also surfaces the
  Round R4 hardening — `trust_domain` immutability on existing Realms,
  cross-domain replay defence, and the warning that rotating the value
  invalidates outstanding `cx.cross_signing.reset` proofs.
- **Added** delivery-binding handover panel rendering the new error codes
  `delivery_binding_stale` / `delivery_binding_handed_over` /
  `historical_only`. Stale entries surface `new_recipient_service_did` and
  `handover_frontier`.
- **Added** 3PID invite admin: the 5-terminal-state machine
  (`claimed` / `send_failed` / `revoked_by_capability_loss` /
  `revoked_by_inviter_left` / `invalidated_by_rate_limit`) is fully visible
  and `send_failed` must render honestly (no fake-success states).
- **Added** account-deactivation panel now shows the
  `deactivation_federation_incomplete` banner state (federation fanout
  cannot complete without a peer ack); silent "fully-deactivated"
  rendering removed.

### Added — Round R2/R3 (contrix-spec rounds 2+3, 2026-05-20)

- **Moderation appeals admin (T06)** — new `/moderation/appeals` route with a
  pending-state list (`submitted` / `under_review`), per-row 30-day auto-close
  countdown, and a detail panel that surfaces the original decision,
  appellant, evidence references, reviewer assignment trail, and decision
  history. Verdict picker writes `cx.moderation.appeal.decision` with the
  Uphold / Overturn / Modify options; Overturn is annotated as auto-pairing
  `cx.moderation.decision.lift` in the same Anchor batch. Separation-of-duties:
  the "Review this appeal" picker is hidden whenever the logged-in admin DID
  equals the issuer of the original moderation decision (mirrors the reducer
  rule "reviewer.did != original_decision.issuer_did").
- **Deactivation 7-domain fanout panel (T07)** — new
  `components::deactivation_fanout_panel` renders the local-PS fanout result
  for `cx.identity.deactivate` across the seven cascade domains (`session`,
  `device`, `applet`, `keypackage`, `push`, `to_device`, `capability`). Failed
  rows render red with a per-domain Retry button. Wired into a new
  `/deactivations/review` page and reused on the realm destroy page below.
- **Realm destroy confirmation dialog (T07)** — new
  `components::realm_destroy_dialog` gates `cx.realm.destroy` behind five
  explicit normative checkboxes (no further ordinary writes, snapshots /
  backfill / GC will run, no successor Realm, erasure receipt + legal hold
  precedence, 30-day federation fanout window) plus a typed-`DESTROY`
  confirmation. Mounted at `/realms/{id}/destroy`, which reuses the fanout
  panel + an erasure-receipt block (local-PS today; cross-PS still
  `TODO(round23-T07)`).
- **Audit attestation evidence admin (T10)** — new `/audit/attestation` route
  with an upload form for `cx.schema.attestation_evidence.v1` JSON documents,
  a list of active rows with validity-remaining chips and `chain_verified` /
  `revocation_checked` status badges. Per-cert chain visualization remains
  `TODO(round23-T10)`.
- **Trust domain deployment setting (T08)** — new `/server/trust-domain` page
  reads/writes the deployment-wide `cx:trust_domain:<scope>` value. Loud red
  warning callout: "Changing trust_domain INVALIDATES every existing
  `cx.cross_signing.reset` proof". The edit field is locked until the admin
  explicitly re-confirms via a checkbox, and validates against the
  `cx:trust_domain:<lowercase-scope>` grammar (≤128 chars after the prefix)
  before submission.
- **Relaxed ephemeral window slider (T09)** — new `/server/relaxed-window`
  page caps the slider at 300_000 ms (the
  `EPHEMERAL_ABSOLUTE_HARD_CEILING_MS` from the SDK) and floors it at 1_000
  ms. When the active deployment profile is `attested_audit.e2ee.v1` or
  `disclosed_audit.e2ee.v1`, the relaxed-profile toggle is grey-disabled with
  a tooltip explaining that audited profiles pin the ephemeral window at the
  protocol hard ceiling.

### Sidebar

- Added nav entries under Moderation (Moderation appeals, Audit attestation)
  and under Server ops (Trust domain, Relaxed window, Deactivation review).

### Backend wiring (placeholders carried forward)

Every new page renders against a deterministic local fixture and emits a
toast on submit. The following `// TODO(round23-T<XX>)` markers track the
real backend wiring still owed by soland / coauth describe surfaces:

- `TODO(round23-T06)` — list + decision POST against
  `/api/admin/v1/moderation/appeals`.
- `TODO(round23-T07)` — describe + retry handlers for
  `/api/admin/v1/identity/deactivations/{id}` and
  `/api/admin/v1/realms/{id}/destroy`; cross-PS erasure receipt
  visualization.
- `TODO(round23-T08)` — `PUT /api/admin/v1/server/trust-domain`.
- `TODO(round23-T09)` — `PUT /api/admin/v1/server/relaxed-window`.
- `TODO(round23-T10)` — `POST /api/admin/v1/audit/attestation-evidence` plus
  the full chain-verification visualization.
