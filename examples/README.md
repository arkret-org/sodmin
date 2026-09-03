# Sodmin examples

The integrated bring-up: postgres + coauth + soland + floria + sodmin,
driven by `docker-compose.example-stack.yaml` + `up.sh` / `down.sh` /
`smoke.sh`. Exercised every night in CI via
[`.github/workflows/nightly-example-stack.yaml`](../.github/workflows/nightly-example-stack.yaml).

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
`arkret/`) so the workspace `Cargo.toml` path-deps
`../arkret-rust-sdk/...` and `../coauth/crates/admin-types` resolve
inside the build sandbox. Soland uses a **named additional context**
(`additional_contexts.arkret-rust-sdk: ../../arkret-rust-sdk`)
because its Dockerfile copies the SDK via `COPY --from=arkret-rust-sdk`.

```
arkret/
  arkret-rust-sdk/                      <-- required (path-dep target)
  coauth/                                 <-- required (path-dep + image)
  soland/
  floria/
  sodmin/
    examples/
      docker-compose.example-stack.yaml   <-- run from here
      up.sh / down.sh / smoke.sh
      coauth-config.yaml                  <-- generated, gitignored
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
- All four sibling repos checked out at `../{arkret-rust-sdk,coauth,soland,floria}`.

Before the first `up.sh` invocation on a new machine, confirm Docker is
reachable and the sibling repos exist. `up.sh` and `smoke.sh` perform the
live stack checks directly.

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
cargo install dioxus-cli@0.7.10 --locked
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

Every `image:` is `${SVC_IMAGE:-svc:dev}`, so you can pin local image tags
instead of rebuilding each service:

```bash
COAUTH_IMAGE=coauth:dev \
SOLAND_IMAGE=soland:dev \
FLORIA_IMAGE=floria:dev \
SODMIN_IMAGE=sodmin:dev \
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
| `cargo chef cook: failed to read /arkret-rust-sdk/...` | Sibling `arkret-rust-sdk` missing from umbrella checkout. Confirm the sibling checkout exists before rerunning. |
| `cargo fetch --locked: ../arkret-rust-sdk not found` | Same as above. The sodmin/coauth Dockerfiles need arkret-rust-sdk side-by-side at the umbrella context root. |
