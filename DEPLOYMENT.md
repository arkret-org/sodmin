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
