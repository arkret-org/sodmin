# sodmin — Release-Readiness Tasks

> Parent plan: [`../_todos_all.md`](../_todos_all.md)
> Project role: admin UI consuming soland + coauth APIs.
> Phase: **3 (track 3b)**.

## State at start (2026-05-24)

- Dioxus 0.7.5 → WebAssembly. 38 admin pages across 68 routes.
- Hand-rolled HTTP clients (36 modules under `src/api/`); generated DTOs in `src/api/generated.rs`.
- i18n: en + zh, 1,700+ keys (`src/utils/i18n.rs`).
- 3 Playwright smoke tests (`dev-mode-banner`, `device-revoke`, `risk-action`).
- 31 TODO/FIXME — most tagged with round-numbers.
- Dockerfile: multi-stage; nginx runtime; CSP/headers hardened in `docker-entrypoint.sh`.

## Phase 3 tasks

### API stub completion (top 10)
- [ ] §1 `src/pages/delivery_binding.rs:178` — wire `round4-delivery-binding-handover-fetch` against soland endpoint.
- [ ] §2 `src/pages/moderation/appeals.rs:22` — replace static dataset with real reducer query (round23-T06).
- [ ] §3 `src/pages/invites_3pid.rs:50,77` — implement `round4-invites-3pid-fetch` + stale re-fetch (round23-T07).
- [ ] §4 `src/pages/realm_destroy.rs:182` — POST handler for destroy confirmation (round23-T07).
- [ ] §5 `src/pages/realm_links.rs:1` — SVG/canvas DAG visualization for realm links (currently list-only).
- [ ] §6 `src/components/deactivation_fanout_panel.rs:103` — cross-PS progress indicator (round23-T07).
- [ ] §7 `src/pages/relaxed_window.rs:143` — PUT handler (round23-T09).
- [ ] §8 `src/pages/trust_domain.rs:180` — PUT handler with rotation confirmation (round23-T08).
- [ ] §9 `src/pages/server_status.rs:381,387` — rate-limit chart + bindings chip view (round4-rate-limit-chart).
- [ ] §10 `src/pages/audit_attestation.rs:178` — POST evidence endpoint + chain/revocation status view (round23-T10).

### API generation (eliminate manual drift)
- [ ] §11 Replace `src/api/coauth.rs` (hand-rolled, 32 KB) with a generated client using `openapi-generator` or `progenitor` against coauth's OpenAPI doc.
- [ ] §12 Same for soland API: consume `/.well-known/contrix/openapi.json` at build time.
- [ ] §13 Update `Cargo.toml` to depend on `coauth-admin-types` directly once §7 of coauth ships.

### Accessibility audit
- [ ] §14 Run an axe-core scan against the deployed SPA; capture findings in `A11Y.md`.
- [ ] §15 Add `role="main"` / proper landmarks to every page layout in `src/components/layout/`.
- [ ] §16 Ensure all form inputs have wrapped `<label>` (audit `src/components/ui/input.rs` and call sites).
- [ ] §17 Verify keyboard-only navigation works for every CRUD flow (devices, peers, audit, policy).
- [ ] §18 Localize date/time formatting via `icu` or `chrono` + Fluent.

### Playwright expansion
- [ ] §19 Add a smoke spec per high-traffic page: dashboard, actors, spaces, federation, moderation, deactivation. Target ≥ 15 specs.
- [ ] §20 Hook the example-stack workflow (`nightly-example-stack.yaml`) into Playwright runs against a live soland+coauth.

### Engineering hygiene (master plan §5)
- [ ] §21 Add Trivy scan to docker workflow.
- [ ] §22 Add local cosign signing evidence for built bundles. Do not create release tags or push signatures.
- [ ] §23 Add Lighthouse CI check (perf + a11y budget).

### Docs
- [ ] §24 Add an architecture diagram to README (sodmin ↔ soland ↔ coauth flows).
- [ ] §25 Add `DEPLOYMENT.md` documenting nginx upstream env vars (`SOLAND_URL`, `COAUTH_URL`, `COAUTH_PUBLIC_URL`, `SODMIN_PORT`).

### Stale concepts
- [ ] §26 Confirm zero leakage of "Place"/"chadex" in user-visible strings (i18n.rs scan + page titles).

## Exit gate (phase 3)

All of:
1. §1-§13 closed (every visible page hits a live endpoint).
2. Playwright suite has ≥ 15 green specs.
3. A11Y audit findings closed or documented as 1.0 deferrals.
4. Record a local `v0.9.0` milestone without creating a git tag.

## Notes

- `bundle.js.map` (216 KB) is committed — confirm it should stay in the repo or move to release artifacts only.
- Today the admin UI assumes `/api/v1/` is stable; once soland adds a v2 endpoint, add an API-version negotiation layer.
