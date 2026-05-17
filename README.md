# sodmin

Contrix administrator web UI for Principal Server and coauth deployments. The app is built with Dioxus and compiled to WebAssembly.

## Scope

- **Dashboard**: server profile, health, storage and conformance status.
- **Actors**: search, detail, devices, sessions, DID/handle and lifecycle status.
- **Spaces**: membership, invites, visibility, policy and destructive actions with audit context.
- **Devices and capabilities**: trust state, key status, grants, delegations and effective permission review.
- **Federation**: peers, service DIDs, transactions, replay/fork quarantine and verification status.
- **Blob/media**: quota, metadata, retention and anti-enumeration diagnostics.
- **coauth**: accounts, sessions, upstream providers, OAuth2 clients, registration tokens, notification channels and audit logs.

`sodmin` does not implement Contrix reducers or authorization decisions. It consumes stable admin API contracts from `soland` and `coauth`.

## Development

```bash
cargo check
cargo test
dx serve --platform web
```

The UI reads `/config.json` at startup. A typical local config is:

```json
{
  "coauth_public_url": "http://localhost:7080"
}
```

## Build

```bash
cargo build --release
dx build --platform web --release
```

## Container

```bash
docker build -t sodmin .
docker run -p 9090:80 \
  -e SOLAND_URL=http://soland:8008 \
  -e COAUTH_URL=http://coauth:7080 \
  -e COAUTH_PUBLIC_URL=https://auth.example.com \
  sodmin
```

## Deployment

The published image bundles a static Dioxus/WASM build behind nginx. At
container start, `docker-entrypoint.sh` reads the runtime environment
and emits `/usr/share/nginx/html/config.json` plus an nginx vhost
configured with sane security headers.

Required and optional environment variables:

| Variable | Required | Purpose | Legacy alias |
| --- | --- | --- | --- |
| `SOLAND_URL` | yes | Internal URL of the soland Principal Server reached by the proxy. Used for `/_palpo/`, `/_matrix/`, `/_synapse/` legacy compatibility. | `PALPO_URL`, `MATRIX_URL` |
| `COAUTH_URL` | recommended | Internal URL of the coauth admin service. Enables the `/auth/`, `/api/v1/auth/`, `/api/admin/`, `/authorize`, `/oauth2/`, `/.well-known/` proxy locations. | `PASION_URL` |
| `COAUTH_PUBLIC_URL` | recommended | Browser-facing coauth origin. Written to `/config.json` for the OAuth2 PKCE redirect. | `PASION_PUBLIC_URL` |
| `SODMIN_PORT` | no | nginx listen port (default `80`). | `PADMIN_PORT` |

Legacy aliases continue to work for one release cycle.

The rendered nginx config sets a tight default Content-Security-Policy
(`default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; ...`),
`X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, and a
`Permissions-Policy` that denies camera/microphone/geolocation/payment.
If the deployment fronts additional origins (e.g. external CDN, third
party auth), edit `connect-src` in `docker-entrypoint.sh` accordingly.

`/config.json` schema:

```json
{
  "coauth_public_url": "https://auth.example.com"
}
```

The Dioxus runtime currently also accepts `pasion_public_url` as an
alias for backwards compatibility. New deployments should use
`coauth_public_url`.

## API Contract

`_restapi.json` is a migration aid only. The target workflow is to generate or validate UI types from:

- `soland` Principal Server Admin OpenAPI.
- `coauth` Auth / Account Admin OpenAPI.

Generated/shared DTOs are consumed through `src/api/generated.rs`; API client code must preserve Contrix error envelopes, reject URL query credentials, propagate `X-Contrix-Request-Id`, send `Idempotency-Key` for mutations, and redact sensitive diagnostics.

## Dashboard Discovery

The dashboard reads native Contrix discovery metadata from `/api/v1/server/describe` and `/api/admin/v1/server/info`. Discovery-backed fields currently rendered include service DID, coauth issuer DID, delegated/public DID resolver endpoint, supported profiles, reducer/schema profiles, event-kind registry version, OpenAPI version, health summary, and conformance declarations.

If discovery is unavailable or an older backend omits a field, the UI renders `-` or `Unknown` and does not treat the profile as implemented.

## Repository Layout

```text
src/
  api/          Contrix/coauth admin API clients
  components/   Shared UI components
  pages/        Route pages
  types/        Shared response/request DTOs
  utils/        i18n, storage, config, errors and diagnostics
e2e/            Playwright smoke and stack tests
examples/       Local deployment examples, pending Contrix stack refresh
```

## Current Gaps

See the cross-project [`../_todos.md`](../_todos.md). Remaining deferred work includes the docs site/user guide, example-stack cold image verification, and cleanup of pre-existing dead-code warnings.
