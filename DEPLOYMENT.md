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
