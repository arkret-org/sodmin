# sodmin Deployment

This document covers local/container deployment and the image contract used by
the Gitea workflow.

## Runtime Model

`sodmin` is a static Dioxus/WASM bundle served by nginx. At container start, `docker-entrypoint.sh` writes `/usr/share/nginx/html/config.json` and renders nginx upstream proxy settings from environment variables.

## Environment Variables

| Variable | Required | Purpose |
| --- | --- | --- |
| `SOLAND_URL` | yes | Internal URL for the soland Principal Server upstream. Requests to `/_cokret/` and the `/_soland/admin/` operator surface proxy here. |
| `COAUTH_URL` | recommended | Internal URL for coauth admin/auth endpoints. Enables `/auth/`, `/_cokret/gate/`, the coauth `/_soland/admin/*` resource roots, `/authorize`, `/oauth2/`, and `/.well-known/` proxy locations. |
| `COAUTH_PUBLIC_URL` | recommended | Browser-facing coauth origin written to `/config.json` for OAuth2 PKCE redirects. |
| `SODMIN_PORT` | no | nginx listen port. Defaults to `80`. |
| `SODMIN_TELEMETRY_ENDPOINT` | no | P5 — opt-in browser-error telemetry sink. When set, `/config.json` exposes the URL and `utils::telemetry` POSTs structured (no-PII) error events. Operator must also flip `localStorage.sodmin_telemetry_opt_in=1`. |

## Port / Path Routing Table

The sodmin container is a single nginx instance serving a Dioxus/WASM SPA
plus reverse-proxy locations for upstream services. P5 — operators
fronting additional services should update both `connect-src` in the
CSP header and the routing table below.

| Browser path | Upstream | Notes |
| --- | --- | --- |
| `/` (SPA fall-through) | sodmin nginx | Serves `index.html`; SPA router takes over. |
| `/healthz` | sodmin nginx | Static liveness probe — serves `index.html` so missing bundle returns 503. |
| `/healthz/deep` | sodmin nginx → `${SOLAND_URL}/healthz` | P5 — readiness probe; returns 503 if soland is unreachable within 2s. |
| `/config.json` | sodmin nginx | Runtime config rendered at boot from env. |
| `/_cokret/self/events/` | `${SOLAND_URL}` | Event ingestion. |
| `/_cokret/self/sync/` | `${SOLAND_URL}` | Sync long-poll. |
| `/_cokret/find/directory/` | `${SOLAND_URL}` | Directory queries. |
| `/_cokret/` (other trust circles) | `${SOLAND_URL}` | Remaining self/root/find/peer/open/edge surface. |
| coauth `/_soland/admin/*` resource roots (accounts, claims, oauth2-sessions, personal-sessions, upstream-oauth-*, user-registration-tokens, connector-health, notification-*, audit-feed, bridge) | `${COAUTH_URL}` | coauth admin endpoints (RBAC enforced server-side); longest-prefix match wins over soland. |
| `/_soland/admin/` (everything else) | `${SOLAND_URL}` | soland operator surface (spaces, moderation, federation, server, media, etc.). |
| `/auth/` and `/_cokret/gate/` | `${COAUTH_URL}` | Token + session endpoints. |
| `/authorize`, `/oauth2/`, `/.well-known/` | `${COAUTH_URL}` | OAuth2 PKCE strand + discovery. |
| `*.wasm`, `*.js`, `*.css`, images | sodmin nginx (`Cache-Control: public, immutable`) | Bundle assets. |

P5 — when `floria` (E2EE / matrix bridge) or `teabay` (developer
console) are fronted alongside sodmin, the recommended pattern is a
separate nginx in front of all three, with sodmin keeping its own
SPA-only fall-through. Inline proxy entries in `docker-entrypoint.sh`
are intentionally minimal: only soland + coauth are first-class
upstreams.

## Local Container Run

```bash
docker build -t sodmin:local .
docker run --rm -p 9090:80 \
  -e SOLAND_URL=http://soland:8008 \
  -e COAUTH_URL=http://coauth:7080 \
  -e COAUTH_PUBLIC_URL=http://localhost:7080 \
  -e SODMIN_PORT=80 \
  sodmin:local
```

## Published Images

The Gitea Docker workflow currently builds and publishes ARM64 images only.
It never publishes an unsuffixed `latest` tag or a multi-architecture manifest.
Use an architecture-suffixed tag until an amd64 build and manifest job exists.

| Registry configuration | Published repository | Published tags |
| --- | --- | --- |
| `REGISTRY_USER` + `REGISTRY_TOKEN` | `<gitea-host>/<owner>/sodmin` | `sha-<sha>-arm64`, `<branch>-arm64`, `latest-arm64` on `main`, version suffixes with `-arm64` on `v*` tags |
| `DOCKERHUB_USER` + `DOCKERHUB_TOKEN` | `<DOCKERHUB_NAMESPACE-or-user>/sodmin` | Same architecture-suffixed tags |
| Pull requests or missing registry credentials | local build only | Not pushed |

The Kubernetes example in `deploy/k8s/deployment.yaml` is therefore pinned to
`kubernetes.io/arch=arm64` and uses `latest-arm64` as the example tag. Replace
the repository with the registry configured for your deployment and pin the
image by digest in production.

## Security Headers

The nginx template sets CSP, `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, and a restrictive `Permissions-Policy`. If an operator fronts additional origins, update `connect-src` in `docker-entrypoint.sh` and keep the change local to that deployment.

The CSP `script-src` directive includes `'wasm-unsafe-eval'` (required
by the Dioxus WASM bundle) but intentionally OMITS `'unsafe-eval'` and
`'unsafe-inline'`. P5 — new agent-UI dialogs use the existing
`ConfirmDialog` / `DangerousActionDialog` Dioxus primitives and do not
introduce any inline-script or eval-based dialog framework; the strict
CSP is preserved.

## Backend RBAC is Canonical

UI "hiding" of buttons and links is a usability affordance, **never** a
security boundary. The backend (soland for admin endpoints, coauth for
auth/identity endpoints) MUST enforce capability checks on every
admin-scoped route, and MUST reject disallowed actions with
`ck.error.capability_denied`. If a sodmin operator forges a request
through DevTools or curl with a stale token, the server-side RBAC layer
is the one that says no.

P5 — the new `GrantedCapabilitiesView` component (rendered above the
agent provision wizard and other destructive forms) shows the
operator's current grant list before submission to reduce the
"click → 403 surprise" loop, but the destructive button is still
visible regardless: only the backend can authoritatively reject the
action.

## CKP-0007 Circle administration

Sodmin's `/circles/*` surfaces (P3A.3) call into soland's `/_cokret/self/circles/*`
admin layer. Before an operator can use those pages, coauth must have
issued the matching `ck.circle.*` capability grants to the operator's
admin DID — typically via the Coauth Capabilities admin page at
`/coauth/capabilities`, or by running the bootstrap migration that
seeds the six actions:

* `ck.circle.create` (medium risk, no constraints)
* `ck.circle.manage` (medium risk, requires `allowed_circle_ids`)
* `ck.circle.member.add` (low risk, no constraints)
* `ck.circle.member.manage` (medium risk, requires `allowed_circle_ids`)
* `ck.circle.member.add.others` (high risk, requires `allowed_circle_ids`)
* `ck.circle.audit` (high risk, paired with `audit_pair_required` check)

Without these grants every Circle admin call returns 403 with
`ck.error.capability_denied`. The sodmin UI surfaces that as
"Administrator capability denied" — coauth side fix.

`connect-src` in `docker-entrypoint.sh` MUST include the soland,
coauth, and (when used) floria origins so the SPA can call them. The
nginx CSP template already covers the default three; add additional
peers per deployment.

## Database migration

sodmin itself is stateless — it ships no database. Operator-facing data
lives in the upstream services (`soland`, `coauth`, `teabay`). Migration
during a sodmin upgrade is therefore a coordination concern, not a
sodmin concern:

1. Run upstream migrations first (`soland`, `coauth`, `teabay`) and
   confirm `/healthz/deep` reports green for each.
2. Roll sodmin forward only after the upstream schema version is
   compatible with the new SPA bundle. The runtime config endpoint
   (`/config.json`) carries the configured upstream URLs; the SPA does
   not pin a schema version, so a backwards-compatible upstream is
   sufficient.
3. If an upstream rollback is required, roll the sodmin SPA back to the
   matching tag first so the SPA never calls a schema it does not know.

## Backup / restore

sodmin holds no persistent data; there is nothing to back up at the
sodmin layer. Operator backup procedures should target:

* `soland` PostgreSQL — daily logical dump (`pg_dump --format=custom`)
  plus 15-minute WAL archive to the operator-owned object store.
* `coauth` PostgreSQL — same cadence; coauth additionally requires the
  admin token rotation table to be included (default).
* `teabay` PostgreSQL — same cadence; directory rebuild from event log
  is possible but slow, so periodic snapshots are preferred.

Restore is upstream-first: bring up the upstream Postgres replica, point
the upstream service at it, verify `/healthz/deep`, then redeploy
sodmin. The SPA picks up the new endpoints on next browser reload via
`/config.json`.

## Disaster recovery

For region-loss scenarios:

1. **DNS failover** — repoint `sodmin.example.com` to the DR region's
   ingress. The SPA is served from any region without state migration.
2. **Upstream failover** — promote the DR-region Postgres replica for
   `soland` / `coauth` / `teabay`; update the sodmin Deployment's
   `SOLAND_URL` / `COAUTH_URL` env vars to point at the DR-region
   upstreams and roll the Deployment.
3. **Capability re-issuance** — coauth admin grants reference an admin
   DID, not a region; existing operator capability rows survive the
   failover. No re-grant cycle is required.
4. **Verification** — load `/healthz/deep` (browser-side it returns 200
   only when `${SOLAND_URL}/healthz` is reachable within 2s) and walk
   the operator smoke-test in `docs/admin-onboarding.md`.

Recovery time objective (RTO) is bounded by the slowest upstream
Postgres promotion. sodmin's own RTO is ~30s (rolling Deployment
replacement); plan for 5-10 minutes end-to-end including DNS TTLs.

## List pagination

All admin list pages (`/agents`, `/applets`, `/devices`, `/spaces`,
`/federation`) use cursor pagination as of Phase 5. The previous
`page + total` model was migrated off because offset pagination forces
the backend into an `O(N)` scan to compute `total` whenever the table
gets large enough to matter; cursor pagination keeps next-page lookup
at `O(1)` (an index seek on the keyset cursor).

The wire shape is a `data` array plus an optional `next_cursor`
string. The SPA maintains a client-side cursor stack so "Previous"
pops back to the prior cursor without re-requesting. Backends that
haven't yet shipped `next_cursor` should simply omit it — the SPA
treats absent `next_cursor` as "this is the last page" and disables
the Next button.
