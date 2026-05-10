#!/usr/bin/env bash
# Bring up the sodmin example stack (postgres + coauth + soland + floria + sodmin).
#
# Usage:
#   ./examples/up.sh                 # build images, generate config, bring stack up
#   ./examples/up.sh --no-build      # reuse existing images
#   ./examples/up.sh --regen-config  # regenerate examples/coauth-config.yaml
#
# Image overrides honour the same env vars the compose file reads:
#   COAUTH_IMAGE  SOLAND_IMAGE  FLORIA_IMAGE  SODMIN_IMAGE
#
# Exit codes:
#   0 — every service reported healthy (or started, for soland) within budget
#   2 — invocation / pre-flight error
#   3 — compose up failed; recent logs are dumped to stderr

set -euo pipefail

EXAMPLES_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="${EXAMPLES_DIR}/docker-compose.example-stack.yaml"
GENERATED_CONFIG="${EXAMPLES_DIR}/coauth-config.yaml"
COAUTH_IMAGE_REF="${COAUTH_IMAGE:-coauth:dev}"

NO_BUILD=0
REGEN_CONFIG=0

for arg in "$@"; do
    case "${arg}" in
        --no-build)     NO_BUILD=1 ;;
        --regen-config) REGEN_CONFIG=1 ;;
        -h|--help)
            sed -n '2,15p' "${BASH_SOURCE[0]}"
            exit 0
            ;;
        *)
            echo "[example-stack/up] unknown flag: ${arg}" >&2
            exit 2
            ;;
    esac
done

if [[ ! -f "${COMPOSE_FILE}" ]]; then
    echo "[example-stack/up] missing ${COMPOSE_FILE}" >&2
    exit 2
fi

if ! command -v docker >/dev/null 2>&1; then
    echo "[example-stack/up] docker not on PATH" >&2
    exit 2
fi

if ! docker info >/dev/null 2>&1; then
    echo "[example-stack/up] docker daemon not reachable (start Docker Desktop / dockerd)" >&2
    exit 2
fi

if ! command -v python3 >/dev/null 2>&1; then
    echo "[example-stack/up] python3 not on PATH; required for coauth config rewrite" >&2
    exit 2
fi

# ─────────────────────────────────────────────────────────────────────
# Step 0: warn about sibling repos. coauth + soland + sodmin share an
# umbrella build context (../..) so the parent dir must contain the
# expected sibling checkouts. Pre-empts the cryptic "no such file or
# directory" cargo chef cook error from C36.5.
# ─────────────────────────────────────────────────────────────────────
UMBRELLA_DIR="$(cd "${EXAMPLES_DIR}/../.." && pwd)"
for sibling in contrix-rust-sdk coauth soland floria sodmin; do
    if [[ ! -d "${UMBRELLA_DIR}/${sibling}" ]]; then
        echo "[example-stack/up] missing sibling ${UMBRELLA_DIR}/${sibling}" >&2
        echo "[example-stack/up] umbrella layout expected: contrix-dev/{contrix-rust-sdk,coauth,soland,floria,sodmin}/" >&2
        exit 2
    fi
done

# ─────────────────────────────────────────────────────────────────────
# Step 1: build (unless --no-build).
#
# We build coauth FIRST and standalone, because step 2 needs to spin
# up an ephemeral coauth container to generate a config — and that
# container must exist before the rest of the stack tries to mount
# `coauth-config.yaml`. Building everything in one shot would still
# work, but isolating coauth surfaces image build errors before the
# slower Rust-heavy soland + sodmin builds start.
# ─────────────────────────────────────────────────────────────────────
if (( NO_BUILD == 0 )); then
    echo "[example-stack/up] building coauth image first (needed for config gen)"
    docker compose -f "${COMPOSE_FILE}" build coauth || {
        echo "[example-stack/up] coauth build failed" >&2
        exit 3
    }
fi

# ─────────────────────────────────────────────────────────────────────
# Step 2: generate coauth config with valid signing keys (idempotent).
#
# Uses an ephemeral coauth container so we don't need to hand-craft
# encryption secrets / RSA + EC keypairs in the repo. The generated
# config is patched via Python (NOT sed — C36.5 found sed-based
# rewrites brittle against multi-listener YAML) to:
#   - point database at the compose-internal postgres
#   - replace the entire `http.listeners:` block with a single
#     web+health listener bound to 0.0.0.0:7080 (so both /health and
#     OAuth surfaces are reachable on one host-published port)
#   - set issuer/public_base to the compose-internal coauth URL
# ─────────────────────────────────────────────────────────────────────
if [[ ! -f "${GENERATED_CONFIG}" ]] || (( REGEN_CONFIG == 1 )); then
    echo "[example-stack/up] generating ${GENERATED_CONFIG} via ephemeral coauth"
    docker run --rm "${COAUTH_IMAGE_REF}" config generate > "${GENERATED_CONFIG}.raw" || {
        echo "[example-stack/up] coauth config generate failed (image=${COAUTH_IMAGE_REF})" >&2
        rm -f "${GENERATED_CONFIG}.raw"
        exit 3
    }

    python3 - "${GENERATED_CONFIG}.raw" "${GENERATED_CONFIG}" <<'PY'
import sys, re, pathlib

src = pathlib.Path(sys.argv[1]).read_text(encoding="utf-8")
out_path = pathlib.Path(sys.argv[2])

# 1. database.uri → compose-internal postgres
src = re.sub(
    r'^(\s*uri:).*$',
    r'\1 postgresql://contrix:contrix@postgres:5432/contrix',
    src, count=1, flags=re.MULTILINE,
)

# 2. http.public_base + http.issuer → in-cluster coauth URL.
#    For the example stack we want sodmin (also in the network) to
#    talk to coauth via service DNS — so the issuer claim matches the
#    URL the SPA uses through the nginx /auth/ proxy.
src = re.sub(
    r'^(\s*public_base:).*$',
    r'\1 http://coauth:7080/',
    src, count=1, flags=re.MULTILINE,
)
src = re.sub(
    r'^(\s*issuer:).*$',
    r'\1 http://coauth:7080/',
    src, count=1, flags=re.MULTILINE,
)

# 3. Replace the entire `http.listeners:` block with ONE web+health
#    listener bound to 0.0.0.0:7080. Default `coauth config generate`
#    emits TWO listeners (`web` on `[::]:7080` minus health + `internal`
#    on `localhost:8091` health-only). Compose only publishes 7080 to
#    the host, so the host /health probe would 404 against `web` and
#    the `internal` listener is unreachable.
#
#    We splice a deterministic block instead of patching in-place so
#    multi-listener YAML doesn't break the rewrite.
LISTENERS_REPLACEMENT = (
    "  listeners:\n"
    "  - name: web\n"
    "    resources:\n"
    "    - name: discovery\n"
    "    - name: human\n"
    "    - name: oauth\n"
    "    - name: compat\n"
    "    - name: restapi\n"
    "    - name: assets\n"
    "    - name: adminapi\n"
    "    - name: health\n"
    "    binds:\n"
    "    - address: '0.0.0.0:7080'\n"
    "    proxy_protocol: false\n"
)

m = re.search(r'^http:\s*$', src, flags=re.MULTILINE)
if not m:
    sys.exit("FATAL: no `http:` section in generated config")
http_start = m.end() + 1

ls = re.search(r'^  listeners:\s*$', src[http_start:], flags=re.MULTILINE)
if not ls:
    sys.exit("FATAL: no `http.listeners:` key in generated config")
ls_abs_start = http_start + ls.start()

# Find the next sibling `  <key>:` line AFTER the listeners block.
# Children of the listeners list begin with `  -` or have deeper
# indent — those stay inside the block. A sibling has an identifier
# character at column 2 (e.g. `  trusted_proxies:`).
lines = src[ls_abs_start:].splitlines(keepends=True)
offset = len(lines[0])  # skip the `  listeners:` line itself
for line in lines[1:]:
    # Top-level key (no leading whitespace) ends the `http:` section.
    if line and not line[0].isspace():
        break
    if (
        len(line) > 2
        and line[0] == " "
        and line[1] == " "
        and line[2].isalpha()
    ):
        break
    offset += len(line)
tail_start = ls_abs_start + offset

src = src[:ls_abs_start] + LISTENERS_REPLACEMENT + src[tail_start:]

out_path.write_text(src, encoding="utf-8")
PY
    rm -f "${GENERATED_CONFIG}.raw"
    echo "[example-stack/up] wrote $(wc -l < "${GENERATED_CONFIG}") line config"
else
    echo "[example-stack/up] reusing existing ${GENERATED_CONFIG} (--regen-config to refresh)"
fi

# ─────────────────────────────────────────────────────────────────────
# Step 3: build the rest (unless --no-build). Done after config gen
# so a build failure here doesn't block the much faster config step.
# ─────────────────────────────────────────────────────────────────────
if (( NO_BUILD == 0 )); then
    echo "[example-stack/up] building remaining images via docker compose build"
    docker compose -f "${COMPOSE_FILE}" build || {
        echo "[example-stack/up] image build failed" >&2
        exit 3
    }
fi

# ─────────────────────────────────────────────────────────────────────
# Step 4: bring the stack up. `--wait` blocks until every healthcheck
# transitions to healthy or budget expires; we then surface log tails
# on failure so CI doesn't have to grep through compose noise.
# ─────────────────────────────────────────────────────────────────────
echo "[example-stack/up] docker compose up -d --wait"
if ! docker compose -f "${COMPOSE_FILE}" up -d --wait; then
    echo "[example-stack/up] compose up failed; dumping recent logs" >&2
    docker compose -f "${COMPOSE_FILE}" ps >&2 || true
    docker compose -f "${COMPOSE_FILE}" logs --tail=120 >&2 || true
    exit 3
fi

cat <<EOF
[example-stack/up] stack is up.

  postgres : localhost:55432   (user=contrix pass=contrix db=contrix)
  coauth   : http://localhost:57080/health
  soland   : http://localhost:58787/health
  floria   : http://localhost:55000/ready  (admin)  +  http://localhost:58000  (data)
  sodmin   : http://localhost:58200/healthz

Run ./examples/smoke.sh to verify each /health returns 200.
Run ./examples/down.sh to tear down + drop volumes.
EOF
