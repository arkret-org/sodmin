# Sodmin examples

This directory holds two parallel docker-compose stacks:

1. **Sodmin example stack** (`docker-compose.example-stack.yaml`) — the
   round-35 integrated bring-up: postgres + coauth + soland + floria +
   sodmin. Driven by `up.sh` / `down.sh` / `smoke.sh`, exercised every
   night in CI via [`.github/workflows/nightly-example-stack.yaml`](../.github/workflows/nightly-example-stack.yaml).
   See [Sodmin example stack](#sodmin-example-stack) below.

2. **Legacy Palpo stack** (`compose.yml`) — older Matrix-homeserver
   demo (palpo + pasion + padmin + element). Kept for the playwright
   `example-stack` smoke suite under `e2e/example-stack/`.
   See [Palpo Stack Example](#palpo-stack-example) further down.

---

## Sodmin example stack

| Service  | Image base                              | Host port | Probe         |
|----------|-----------------------------------------|-----------|---------------|
| postgres | `postgres:16-alpine`                    | `55432`   | `pg_isready`  |
| coauth   | `gcr.io/distroless/cc-debian12:nonroot` | `57080`   | `/health`     |
| soland   | `debian:bookworm-slim`                  | `58787`   | `/health`     |
| floria   | `debian:bookworm-slim` (+ curl)         | `55000`   | `/ready`      |
| sodmin   | `nginx:alpine`                          | `58200`   | `/healthz`    |

Container ports differ from host ports; see
[`docker-compose.example-stack.yaml`](docker-compose.example-stack.yaml)
for the full mapping.

### Repository layout assumption

The compose file references sibling repos via relative build contexts.
Coauth + sodmin both use an **umbrella build context** (`../..` →
`contrix-dev/`) so the workspace `Cargo.toml` path-deps
`../contrix-rust-sdk/...` and `../coauth/crates/admin-types` resolve
inside the build sandbox. Soland uses a **named additional context**
(`additional_contexts.contrix-rust-sdk: ../../contrix-rust-sdk`)
because its Dockerfile copies the SDK via `COPY --from=contrix-rust-sdk`.

```
contrix-dev/
  contrix-rust-sdk/                      <-- required (path-dep target)
  coauth/                                 <-- required (path-dep + image)
  soland/
  floria/
  sodmin/
    examples/
      docker-compose.example-stack.yaml   <-- run from here
      up.sh / down.sh / smoke.sh
      coauth-config.yaml                  <-- generated, gitignored
    scripts/
      preflight-check.sh                  <-- run before up.sh on a new machine
```

If your checkout differs, override every `*_IMAGE` env var with a
prebuilt tag (see "Image overrides" below) and skip the `--build` step.

### Operator pre-requisites

The compose stack expects:

- Docker Desktop / dockerd reachable (`docker info` returns 0).
- `python3` on PATH (`up.sh` uses Python to rewrite the generated
  `coauth-config.yaml` deterministically — sed-based YAML edits proved
  brittle against the multi-listener default config in C36.5).
- `curl` on PATH (`smoke.sh` probes each `/health` endpoint).
- All four sibling repos checked out at `../{contrix-rust-sdk,coauth,soland,floria}`.

Run `./scripts/preflight-check.sh` before the first `up.sh` invocation
on a new machine — it surfaces every misconfig class that has burnt
15+ minutes of cold Rust build in earlier rounds (port collisions,
distroless-incompatible probes, missing siblings, missing host tools).

### Manual binary builds (advanced operators)

If you want to dogfood a service without docker, build each binary
locally with the same feature flags the Dockerfiles use:

```bash
# coauth — distroless build features (no native bundling); wired in C36.5
cargo build --bin coauth --no-default-features --features docker,cedar -p coauth

# soland — single-binary release; default features
cargo build --release --bin soland

# floria — single-binary release; default features
cargo build --release --bin floria

# sodmin — Dioxus WASM SPA; uses dx, not cargo
cargo install dioxus-cli@0.7.5 --locked
dx build --release   # output under target/dx/sodmin/release/web/public
```

Smoke binary (Rust-side): `cargo build --bin sodmin-smoke` builds the
post-deploy probe shipped in the sodmin repo (`src/bin/sodmin_smoke.rs`).

### Quickstart

```bash
# 1. Bring the stack up (builds images + generates config + waits for /health)
./examples/up.sh

# 2. Confirm every service is reachable
./examples/smoke.sh

# 3. Tear down + drop volumes
./examples/down.sh
```

Open [http://localhost:58200](http://localhost:58200) for the sodmin SPA
once `up.sh` reports the stack healthy.

### Healthcheck strategy

The sodmin team locked this in during C33.6 (coauth) and C35.3 (this
work):

- **distroless images** (coauth) get a `CMD`-form healthcheck that
  calls a `<bin> healthcheck` subcommand baked into the binary.
  Distroless has no shell, no `wget`, no `curl` — `CMD-SHELL`-form
  probes silently degrade to "always unhealthy" against them.
- **debian-slim runtimes** (floria) ship `curl` explicitly and probe
  HTTP directly.
- **alpine runtimes** (sodmin via nginx:alpine) probe with `wget
  --spider` because alpine ships busybox-wget by default.
- **soland** runs on debian-slim without curl yet; the compose file
  treats its TCP listener as readiness (no in-container probe), and
  `smoke.sh` does the actual `/health` HTTP probe externally. Adding
  curl to the soland Dockerfile or a `soland healthcheck` subcommand
  is the next step.

### Image overrides

Every `image:` is `${SVC_IMAGE:-svc:dev}`, so you can pin published
nightly tags instead of building locally:

```bash
COAUTH_IMAGE=ghcr.io/contrix/coauth:nightly \
SOLAND_IMAGE=ghcr.io/contrix/soland:nightly \
FLORIA_IMAGE=ghcr.io/contrix/floria:nightly \
SODMIN_IMAGE=ghcr.io/contrix/sodmin:nightly \
    ./examples/up.sh --no-build
```

### Coauth config generation

`up.sh` generates `examples/coauth-config.yaml` on first run by exec'ing
`coauth config generate` inside an ephemeral coauth container. This
gives you a valid config with fresh signing keys and an encryption
secret without committing keys to the repo. Use `--regen-config` to
refresh it.

### Nightly CI

[`.github/workflows/nightly-example-stack.yaml`](../.github/workflows/nightly-example-stack.yaml)
runs `up.sh` + `smoke.sh` on a fresh runner every night at 07:30 UTC
(plus on PRs that touch the example stack files). On failure it uploads
container logs as artifacts (`example-stack-logs-<run_id>`) so a
regression doesn't require local docker reproduction.

### Troubleshooting

| Symptom                                 | Likely cause / fix                                          |
|-----------------------------------------|-------------------------------------------------------------|
| `coauth config generate failed`         | `coauth:dev` image not built — run `up.sh` without `--no-build`, or pre-pull `COAUTH_IMAGE`. |
| `port 55432 already bound`              | Local postgres on host. Stop it or change the host port in the compose file. |
| `coauth` healthcheck flaps              | Postgres still migrating. The `start_period: 20s` should cover it; bump if your machine is slow. |
| `sodmin: never returned 200`            | The Dioxus build inside the image is large. Re-run `smoke.sh` with `SMOKE_ATTEMPT_BUDGET=120`. |
| Build context error: `../../coauth` not found | Sibling repos aren't checked out. Either clone them next to `sodmin/` or override every `*_IMAGE` env var. |
| `502 Bad Gateway` during apt-get      | Docker Desktop proxy or upstream Debian mirror flake. Floria/coauth wrap apt in a 5-attempt retry loop (added C37.3); just re-run `up.sh` if it surfaces. |
| `cargo chef cook: failed to read /contrix-rust-sdk/...` | Sibling `contrix-rust-sdk` missing from umbrella checkout. Run `./scripts/preflight-check.sh` to confirm. |
| `cargo fetch --locked: ../contrix-rust-sdk not found` | Same as above. The sodmin/coauth Dockerfiles need contrix-rust-sdk side-by-side at the umbrella context root. |

---

## Palpo Stack Example

A complete local development stack running a Matrix homeserver with OAuth/OIDC authentication, an admin dashboard, and Element Web client.

## Services

| Service | Description | Host Port | Internal Port |
|---------|-------------|-----------|---------------|
| **postgres** | PostgreSQL database (shared by palpo and pasion) | `15432` | `5432` |
| **palpo** | Matrix homeserver (Client-Server API) | `8008`, `8448` | `8008`, `8448` |
| **pasion** | OAuth 2.0 / OpenID Connect authentication service | `7080` | `7080` |
| **padmin** | Admin dashboard web UI (nginx + Dioxus WASM) | `7060` | `80` |
| **element** | Element Web Matrix client | `7070` | `80` |

## Quick Start

```bash
# 1. Review and adjust configuration files as needed
#    - palpo.toml    (Matrix homeserver config)
#    - pasion.yaml   (OAuth/OIDC config)
#    - element-config.json (Element Web config)

# 2. Build and start all services
docker compose up -d --build

# 3. Open the services in your browser
#    Admin dashboard:  http://localhost:7060
#    Auth service:     http://localhost:7080
#    Element client:   http://localhost:7070
#    Matrix API:       http://localhost:8008
```

## Smoke Tests

The example-stack smoke tests now live in the root Playwright workspace rather
than under `examples/e2e/`.

Run them from the repository root:

```bash
npm install
npx playwright install chromium
npm run test:example-stack:fresh
```

If you need to inspect the reset flow separately:

```bash
npm run stack:reset
npm run test:example-stack
```

The smoke-suite implementation and caveats are documented in
[`../e2e/example-stack/README.md`](../e2e/example-stack/README.md).

## Architecture

```
Browser
  |
  +---> :7070  Element Web  ----+
  +---> :7080  Pasion (Auth) ---+--> :8008 Palpo (Matrix) --> :5432 PostgreSQL
  +---> :7060  Padmin (Admin) --+                                    |
                                                                     |
         Pasion (Auth) --------------------------------------------->+
```

- **Palpo** is the Matrix homeserver handling Client-Server API and federation.
- **Pasion** provides OAuth 2.0 / OIDC authentication. Palpo delegates auth to Pasion via `delegated_auth` in `palpo.toml`.
- **Padmin** is a static Dioxus WASM app served by nginx for managing the homeserver.
- **Element** is the standard Matrix web client, preconfigured to connect to the local Palpo instance.
- **PostgreSQL** hosts two databases: `palpo` (homeserver data) and `pasion` (auth data), initialized by `init-db.sh`.

## Configuration Files

| File | Purpose |
|------|---------|
| `palpo.toml` | Palpo homeserver settings: server name, database, federation, delegated auth |
| `pasion.yaml` | Pasion OAuth/OIDC: database, Matrix integration, password policy, upstream providers, email |
| `pasion-signing-key.pem` | Signing key for Pasion JWT tokens |
| `element-config.json` | Element Web client config: homeserver URL and server name |
| `init-db.sh` | PostgreSQL entrypoint script that creates the `pasion` database |
| `nginx.conf` | Reference nginx config for serving Dioxus WASM apps (not mounted by default) |

## Database

- **User**: `palpo`
- **Password**: `changeme`
- **Host**: `localhost:15432` (from host) or `postgres:5432` (from containers)
- **Databases**: `palpo`, `pasion`

Connect from host:

```bash
psql -h localhost -p 15432 -U palpo -d palpo
psql -h localhost -p 15432 -U palpo -d pasion
```

## Default Credentials and Secrets

> **Warning**: These are development defaults. Change them before any non-local deployment.

| Setting | Value | File |
|---------|-------|------|
| Postgres password | `changeme` | `compose.yml`, `palpo.toml`, `pasion.yaml` |
| MAS shared secret | `replace-with-a-random-secret` | `palpo.toml`, `pasion.yaml` |
| Encryption key | `0a1b2c...` (hex string) | `pasion.yaml` |

## Upstream OAuth Providers

Pasion is preconfigured with a GitHub OAuth provider. To use it:

1. Create a GitHub OAuth App at <https://github.com/settings/developers>.
2. Set the **Authorization callback URL** to:
   ```
   http://localhost:7080/upstream/callback/<provider_id>
   ```
   where `<provider_id>` matches the `id` field of the provider in `pasion.yaml` (e.g. `01KMQDVNFWTRF9K8FV8FCFKARM`).
3. Update `client_id` and `client_secret` in `pasion.yaml` with your app's credentials.

> **Note**: The `redirect_uris` for the `Palpo Admin Dashboard` client in `pasion.yaml` must match the host port padmin is served on (`http://localhost:7060/oauth/callback`), not the Pasion port.

## Volumes

| Volume | Purpose |
|--------|---------|
| `postgres_data` | PostgreSQL data directory |
| `palpo_media` | Uploaded media files |

To reset all data:

```bash
docker compose down -v
```

## Useful Commands

```bash
# View logs for a specific service
docker compose logs -f pasion

# Rebuild a single service
docker compose build padmin

# Restart a single service
docker compose restart palpo

# Stop everything
docker compose down

# Stop and remove all data
docker compose down -v
```
