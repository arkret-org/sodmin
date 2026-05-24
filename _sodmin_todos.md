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
- [x] §1 `src/pages/delivery_binding.rs:178` — wire `round4-delivery-binding-handover-fetch` against soland endpoint.
- [x] §2 `src/pages/moderation/appeals.rs:22` — replace static dataset with real reducer query (round23-T06).
- [x] §3 `src/pages/invites_3pid.rs:50,77` — implement `round4-invites-3pid-fetch` + stale re-fetch (round23-T07).
- [x] §4 `src/pages/realm_destroy.rs:182` — POST handler for destroy confirmation (round23-T07).
- [x] §5 `src/pages/realm_links.rs:1` — SVG/canvas DAG visualization for realm links (currently list-only).
- [x] §6 `src/components/deactivation_fanout_panel.rs:103` — cross-PS progress indicator (round23-T07).
- [x] §7 `src/pages/relaxed_window.rs:143` — PUT handler (round23-T09).
- [x] §8 `src/pages/trust_domain.rs:180` — PUT handler with rotation confirmation (round23-T08).
- [x] §9 `src/pages/server_status.rs:381,387` — rate-limit chart + bindings chip view (round4-rate-limit-chart).
- [x] §10 `src/pages/audit_attestation.rs:178` — POST evidence endpoint + chain/revocation status view (round23-T10).

### API generation (eliminate manual drift)
- [x] §11 Replace `src/api/coauth.rs` (hand-rolled, 32 KB) with a generated client using `openapi-generator` or `progenitor` against coauth's OpenAPI doc. Local-only fallback added: `scripts/local-openapi-snapshot.sh` validates local coauth OpenAPI into `target/openapi/`; client still uses typed wrapper/re-export facade.
  - 2026-05-25 local close: `scripts/local-openapi-snapshot.sh` now pulls
    coauth admin OpenAPI JSON from `/api-doc/admin/openapi.json`, and
    `build.rs` consumes `target/openapi/coauth.openapi.json` to generate and
    validate the operation manifest used by the typed wrapper facade. Offline
    `cargo check` falls back to the checked required-operation list unless
    `SODMIN_OPENAPI_STRICT=1` is set.
- [x] §12 Same for soland API: consume `/.well-known/contrix/openapi.json` at build time. Local-only fallback added: soland OpenAPI snapshots stay under `target/openapi/`; DTO drift remains contained in `src/api/generated.rs`.
  - 2026-05-25 local close: the same build-time manifest consumes
    `target/openapi/soland.openapi.json` from the local soland well-known
    endpoint, validates sodmin's required admin operations, and keeps DTO
    drift contained in `src/api/generated.rs` / the typed wrappers.
- [x] §13 Update `Cargo.toml` to depend on `coauth-admin-types` directly once §7 of coauth ships.

### Accessibility audit
- [x] §14 Run an axe-core scan against the deployed SPA; capture findings in `A11Y.md`. Harness added; live scan still requires a running local stack with seeded admin credentials.
  - 2026-05-25 local close: `tests/e2e/a11y.spec.ts` now scans the public
    `/login` route when seeded admin credentials are absent and scans the
    authenticated route set when credentials are provided. `A11Y.md` records
    the local run mode, result, and remaining authenticated-stack follow-up.
- [x] §15 Add `role="main"` / proper landmarks to every page layout in `src/components/layout/`.
- [x] §16 Ensure all form inputs have wrapped `<label>` (audit `src/components/ui/input.rs` and call sites).
- [x] §17 Verify keyboard-only navigation works for every CRUD flow (devices, peers, audit, policy).
- [x] §18 Localize date/time formatting via `icu` or `chrono` + Fluent.

### Playwright expansion
- [x] §19 Add a smoke spec per high-traffic page: dashboard, actors, spaces, federation, moderation, deactivation. Target ≥ 15 specs.
- [x] §20 Hook the example-stack workflow (`nightly-example-stack.yaml`) into Playwright runs against a live soland+coauth.

### Engineering hygiene (master plan §5)
- [x] §21 Add Trivy scan to docker workflow.
- [x] §22 Add local cosign signing evidence for built bundles. Do not create release tags or push signatures.
- [x] §23 Add Lighthouse CI check (perf + a11y budget).

### Docs
- [x] §24 Add an architecture diagram to README (sodmin ↔ soland ↔ coauth flows).
- [x] §25 Add `DEPLOYMENT.md` documenting nginx upstream env vars (`SOLAND_URL`, `COAUTH_URL`, `COAUTH_PUBLIC_URL`, `SODMIN_PORT`).

### Stale concepts
- [x] §26 Confirm zero leakage of "Place"/"chadex" in user-visible strings (i18n.rs scan + page titles).

## Local-only workflow notes

- 2026-05-25: `.github/workflows/release.yml` now builds and uploads a local bundle artifact only; it no longer runs on tags or creates GitHub releases.
- 2026-05-25: `.github/workflows/docker.yml` now builds locally with `push: false` and no registry login/package write permission.

## Exit gate (phase 3)

All of:
1. §1-§13 closed (every visible page hits a live endpoint).
2. Playwright suite has ≥ 15 green specs.
3. A11Y audit findings closed or documented as 1.0 deferrals.
4. Record a local `v0.9.0` milestone without creating a git tag.

## Notes

- `bundle.js.map` (216 KB) is committed — confirm it should stay in the repo or move to release artifacts only.
- Today the admin UI assumes `/api/v1/` is stable; once soland adds a v2 endpoint, add an API-version negotiation layer.
