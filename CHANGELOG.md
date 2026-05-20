# Changelog

All notable changes to `sodmin` (Contrix Server admin UI) are documented in
this file. Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/);
this project is pre-1.0 so wire-breaking changes can land in any release until
the SDK pins down its 1.0 contract.

## [Unreleased]

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
