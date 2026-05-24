#!/usr/bin/env bash
set -euo pipefail

soland_url="${SOLAND_URL:-http://localhost:58787}"
coauth_url="${COAUTH_URL:-http://localhost:57080}"
out_dir="${1:-target/openapi}"

mkdir -p "$out_dir"

fetch() {
  local name="$1"
  local url="$2"
  local out="$3"

  echo "fetching ${name}: ${url}"
  curl --fail --silent --show-error "$url" -o "$out"
  jq empty "$out" >/dev/null
}

fetch "soland" "${soland_url%/}/.well-known/contrix/openapi.json" "${out_dir}/soland.openapi.json"
fetch "coauth" "${coauth_url%/}/.well-known/contrix/openapi.json" "${out_dir}/coauth.openapi.json"

cat > "${out_dir}/README.md" <<EOF
# Local OpenAPI snapshots

Generated from local services only.

- soland: \`${soland_url%/}/.well-known/contrix/openapi.json\`
- coauth: \`${coauth_url%/}/.well-known/contrix/openapi.json\`

These snapshots are intentionally written under \`target/openapi\` and
must not be committed as generated client code. Sodmin currently consumes
shared DTOs through \`src/api/generated.rs\`, with coauth types re-exported
from \`coauth-admin-types\`.
EOF
