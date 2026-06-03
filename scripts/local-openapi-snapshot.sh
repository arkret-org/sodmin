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

fetch "soland" "${soland_url%/}/.well-known/cokret/openapi.json" "${out_dir}/soland.openapi.json"

coauth_admin_tmp="${out_dir}/coauth.admin.openapi.json.tmp"
coauth_account_tmp="${out_dir}/coauth.account.openapi.json.tmp"
fetch "coauth-admin" "${coauth_url%/}/api-doc/admin/openapi.json" "$coauth_admin_tmp"
fetch "coauth-account" "${coauth_url%/}/api-doc/openapi.json" "$coauth_account_tmp"
jq -s '.[0] + {paths: ((.[0].paths // {}) + (.[1].paths // {}))}' \
  "$coauth_admin_tmp" \
  "$coauth_account_tmp" \
  > "${out_dir}/coauth.openapi.json"
jq empty "${out_dir}/coauth.openapi.json" >/dev/null
rm -f "$coauth_admin_tmp" "$coauth_account_tmp"

cat > "${out_dir}/README.md" <<EOF
# Local OpenAPI snapshots

Generated from local services only.

- soland: \`${soland_url%/}/.well-known/cokret/openapi.json\`
- coauth-admin: \`${coauth_url%/}/api-doc/admin/openapi.json\`
- coauth-account: \`${coauth_url%/}/api-doc/openapi.json\`

These snapshots are intentionally written under \`target/openapi\` and
must not be committed as generated client code. \`build.rs\` consumes
them to generate and validate \`sodmin_openapi_contracts.rs\` under
\`OUT_DIR\`; typed wrappers keep DTO ownership in \`src/api/generated.rs\`
and \`coauth-admin-types\`.
EOF
