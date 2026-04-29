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
docker run -p 9090:80 sodmin
```

## API Contract

`_restapi.json` is a migration aid only. The target workflow is to generate or validate UI types from:

- `soland` Principal Server Admin OpenAPI.
- `coauth` Auth / Account Admin OpenAPI.

Until generated contracts are wired in, API client code must preserve Contrix error envelopes, reject URL query credentials, propagate `X-Contrix-Request-Id`, send `Idempotency-Key` for mutations, and redact sensitive diagnostics.

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

See [`_todos.md`](./_todos.md). Remaining P0 work includes generated contract source-of-truth, full coauth account pages, high-risk action approval UX, and a local Contrix compose stack replacing legacy fixtures.
