#!/usr/bin/env bash
# Preflight check for the sodmin example-stack docker-compose bring-up.
#
# Catches misconfiguration classes that would otherwise burn 15+ minutes
# of cold Rust build before failing:
#
#   1. docker / docker compose presence + daemon reachability
#   2. host tools (python3, curl) needed by up.sh / smoke.sh
#   3. sibling repo layout (contrix-dev/{contrix-rust-sdk,coauth,soland,floria,sodmin})
#   4. compose-file syntactic validity
#   5. host port collisions (55432, 57080, 58787, 55000, 58000, 58200)
#   6. distroless-incompatible healthchecks (CMD-SHELL / wget / curl
#      against any service whose runtime base is documented as distroless)
#   7. Dockerfile subcommand sanity (coauth healthcheck/server/config exist)
#   8. dependent images (postgres pulled, dev tags reachable or compose
#      has `build:` clause)
#
# Exit codes:
#   0 — every check passed
#   1 — at least one warning (best-effort fixable)
#   2 — at least one hard error (would break compose up)
#
# Usage:
#   ./scripts/preflight-check.sh
#   ./scripts/preflight-check.sh --strict     # warnings become errors
#
# Adapted from coauth/scripts/preflight-check.sh (C28-30 work) for the
# sodmin example-stack layout introduced in C35.3 / hardened in C37.3.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
EXAMPLES_DIR="${REPO_ROOT}/examples"
COMPOSE_FILE="${EXAMPLES_DIR}/docker-compose.example-stack.yaml"
UMBRELLA_DIR="$(cd "${REPO_ROOT}/.." && pwd)"
COAUTH_CLI_MOD_RS="${UMBRELLA_DIR}/coauth/crates/cli/src/commands/mod.rs"

STRICT=0
for arg in "$@"; do
    case "${arg}" in
        --strict) STRICT=1 ;;
        -h|--help) sed -n '2,30p' "${BASH_SOURCE[0]}"; exit 0 ;;
        *) echo "[preflight] unknown flag: ${arg}" >&2; exit 2 ;;
    esac
done

WARNINGS=0
ERRORS=0

ok()    { echo "[preflight]   OK   $*"; }
warn()  { echo "[preflight]   WARN $*" >&2; WARNINGS=$((WARNINGS + 1)); }
err()   { echo "[preflight]   ERR  $*" >&2; ERRORS=$((ERRORS + 1)); }
heading() { echo; echo "[preflight] === $* ==="; }

# ─────────────────────────────────────────────────────────────────────
# 1. docker availability
# ─────────────────────────────────────────────────────────────────────
heading "docker"
if ! command -v docker >/dev/null 2>&1; then
    err "docker not found on PATH"
else
    ok "docker $(docker --version | awk '{print $3}' | tr -d ',')"
    if ! docker info >/dev/null 2>&1; then
        err "docker daemon not reachable (is Docker Desktop / dockerd running?)"
    else
        ok "docker daemon reachable"
    fi
fi
if ! docker compose version >/dev/null 2>&1; then
    err "docker compose plugin not installed"
else
    ok "docker compose $(docker compose version --short 2>/dev/null || echo unknown)"
fi

# ─────────────────────────────────────────────────────────────────────
# 2. host tools
# ─────────────────────────────────────────────────────────────────────
heading "host tools"
if command -v python3 >/dev/null 2>&1; then
    ok "python3 $(python3 --version 2>&1 | awk '{print $2}')"
else
    err "python3 not on PATH; up.sh config rewrite will fail"
fi
if command -v curl >/dev/null 2>&1; then
    ok "curl present"
else
    err "curl not on PATH; smoke.sh probes will fail"
fi

# ─────────────────────────────────────────────────────────────────────
# 3. sibling repo layout
#
# Every Rust service builds from an umbrella context (`../..` →
# contrix-dev/) so the workspace `Cargo.toml` path-deps
# `../contrix-rust-sdk/...` etc. resolve in the build sandbox.
# ─────────────────────────────────────────────────────────────────────
heading "sibling repos"
for sibling in contrix-rust-sdk coauth soland floria sodmin; do
    if [[ -d "${UMBRELLA_DIR}/${sibling}" ]]; then
        ok "${sibling} present at ${UMBRELLA_DIR}/${sibling}"
    else
        err "missing sibling ${UMBRELLA_DIR}/${sibling}; build will fail"
    fi
done

# ─────────────────────────────────────────────────────────────────────
# 4. compose file validity
# ─────────────────────────────────────────────────────────────────────
heading "compose file"
if [[ ! -f "${COMPOSE_FILE}" ]]; then
    err "missing ${COMPOSE_FILE}"
else
    if docker compose -f "${COMPOSE_FILE}" config --quiet 2>compose.err; then
        ok "compose file parses"
    else
        err "compose file invalid:"
        cat compose.err >&2
    fi
    rm -f compose.err
fi

# ─────────────────────────────────────────────────────────────────────
# 5. port collisions
# ─────────────────────────────────────────────────────────────────────
heading "host port collisions"
declare -a HOST_PORTS=()
while IFS= read -r port; do
    [[ -n "${port}" ]] && HOST_PORTS+=("${port}")
done < <(grep -oE '"[0-9]+:[0-9]+"' "${COMPOSE_FILE}" | tr -d '"' | cut -d: -f1 | sort -u)

for port in "${HOST_PORTS[@]}"; do
    bound=0
    if command -v ss >/dev/null 2>&1; then
        ss -ltn 2>/dev/null | awk '{print $4}' | grep -q ":${port}\$" && bound=1
    elif command -v netstat >/dev/null 2>&1; then
        netstat -an 2>/dev/null | grep -E "[:.]${port}[[:space:]].*LISTEN" >/dev/null && bound=1
    fi
    if [[ "${bound}" == "1" ]]; then
        warn "host port ${port} already bound; compose up will fail"
    else
        ok "host port ${port} is free"
    fi
done

# ─────────────────────────────────────────────────────────────────────
# 6. distroless-incompatible healthchecks
# ─────────────────────────────────────────────────────────────────────
heading "healthcheck portability"
if grep -nE 'CMD-SHELL.*(wget|curl)' "${COMPOSE_FILE}" >/dev/null; then
    err "compose file has CMD-SHELL wget/curl healthcheck — incompatible with distroless"
    grep -nE 'CMD-SHELL.*(wget|curl)' "${COMPOSE_FILE}" | sed 's/^/      /' >&2
else
    ok "no CMD-SHELL wget/curl healthchecks (distroless-safe)"
fi

# ─────────────────────────────────────────────────────────────────────
# 7. coauth subcommands referenced by compose healthcheck
# ─────────────────────────────────────────────────────────────────────
heading "coauth subcommands"
if [[ -f "${COAUTH_CLI_MOD_RS}" ]]; then
    if grep -q 'Healthcheck' "${COAUTH_CLI_MOD_RS}"; then
        ok "coauth healthcheck subcommand wired"
    else
        err "coauth healthcheck subcommand missing from ${COAUTH_CLI_MOD_RS}"
    fi
    if grep -q 'Server' "${COAUTH_CLI_MOD_RS}" && grep -q 'Config' "${COAUTH_CLI_MOD_RS}"; then
        ok "coauth server + config subcommands present"
    else
        err "coauth server / config subcommands missing"
    fi
else
    warn "${COAUTH_CLI_MOD_RS} not found (coauth sibling missing?)"
fi

# ─────────────────────────────────────────────────────────────────────
# 8. dependent images
# ─────────────────────────────────────────────────────────────────────
heading "dependent images"
if docker image inspect postgres:16-alpine >/dev/null 2>&1; then
    ok "postgres:16-alpine present locally"
else
    warn "postgres:16-alpine not pulled (compose up will pull it)"
fi
for img in coauth:dev soland:dev floria:dev sodmin:dev; do
    if docker image inspect "${img}" >/dev/null 2>&1; then
        ok "${img} present locally"
    else
        warn "${img} not present locally; will be built by compose up"
    fi
done

# ─────────────────────────────────────────────────────────────────────
# Verdict
# ─────────────────────────────────────────────────────────────────────
echo
echo "[preflight] ─── summary ─── errors=${ERRORS} warnings=${WARNINGS}"

if (( ERRORS > 0 )); then
    exit 2
fi
if (( WARNINGS > 0 )) && (( STRICT == 1 )); then
    exit 1
fi
exit 0
