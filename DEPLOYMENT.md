# sodmin Deployment

This document covers local/container deployment only. The project does not push images, create tags, or publish release assets as part of the local readiness workflow.

## Runtime Model

`sodmin` is a static Dioxus/WASM bundle served by nginx. At container start, `docker-entrypoint.sh` writes `/usr/share/nginx/html/config.json` and renders nginx upstream proxy settings from environment variables.

## Environment Variables

| Variable | Required | Purpose |
| --- | --- | --- |
| `SOLAND_URL` | yes | Internal URL for the soland Principal Server upstream. Requests to `/api/v1/` and soland admin routes proxy here. |
| `COAUTH_URL` | recommended | Internal URL for coauth admin/auth endpoints. Enables `/auth/`, `/api/v1/auth/`, `/api/admin/`, `/authorize`, `/oauth2/`, and `/.well-known/` proxy locations. |
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
| `/api/v1/events/` | `${SOLAND_URL}` | Event ingestion. |
| `/api/v1/sync/` | `${SOLAND_URL}` | Sync long-poll. |
| `/api/v1/directory/` | `${SOLAND_URL}` | Directory queries. |
| `/api/admin/` | `${COAUTH_URL}` | All coauth admin endpoints (RBAC enforced server-side). |
| `/auth/` and `/api/v1/auth/` | `${COAUTH_URL}` | Token + session endpoints. |
| `/authorize`, `/oauth2/`, `/.well-known/` | `${COAUTH_URL}` | OAuth2 PKCE flow + discovery. |
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
`cx.error.capability_denied`. If a sodmin operator forges a request
through DevTools or curl with a stale token, the server-side RBAC layer
is the one that says no.

P5 — the new `GrantedCapabilitiesView` component (rendered above the
agent provision wizard and other destructive forms) shows the
operator's current grant list before submission to reduce the
"click → 403 surprise" loop, but the destructive button is still
visible regardless: only the backend can authoritatively reject the
action.

## CXP-0007 Circle administration

Sodmin's `/circles/*` surfaces (P3A.3) call into soland's `/api/v1/circles/*`
admin layer. Before an operator can use those pages, coauth must have
issued the matching `cx.circle.*` capability grants to the operator's
admin DID — typically via the Coauth Capabilities admin page at
`/coauth/capabilities`, or by running the bootstrap migration that
seeds the six actions:

* `cx.circle.create` (medium risk, no constraints)
* `cx.circle.manage` (medium risk, requires `allowed_circle_refs`)
* `cx.circle.member.add` (low risk, no constraints)
* `cx.circle.member.manage` (medium risk, requires `allowed_circle_refs`)
* `cx.circle.member.add.others` (high risk, requires `allowed_circle_refs`)
* `cx.circle.audit` (high risk, paired with `audit_pair_required` check)

Without these grants every Circle admin call returns 403 with
`cx.error.capability_denied`. The sodmin UI surfaces that as
"Administrator capability denied" — coauth side fix.

`connect-src` in `docker-entrypoint.sh` MUST include the soland,
coauth, and (when used) floria origins so the SPA can call them. The
nginx CSP template already covers the default three; add additional
peers per deployment.

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
