#!/usr/bin/env bash
set -euo pipefail

artifact="${1:-}"
out_dir="${2:-cosign-evidence}"

if [[ -z "$artifact" || ! -f "$artifact" ]]; then
  echo "usage: $0 <artifact> [out-dir]" >&2
  exit 2
fi

mkdir -p "$out_dir"

base="$(basename "$artifact")"
sha_file="${out_dir}/${base}.sha256"
bundle_file="${out_dir}/${base}.cosign.bundle"

sha256sum "$artifact" > "$sha_file"

cat > "${out_dir}/README.md" <<EOF
# Local cosign evidence

Artifact: \`${base}\`
Generated: \`$(date -u +"%Y-%m-%dT%H:%M:%SZ")\`

This directory is local build evidence only. It is not a release tag,
registry signature, GitHub release, or crates.io publication.

Verify checksum:

\`\`\`bash
sha256sum -c ${base}.sha256
\`\`\`
EOF

if command -v cosign >/dev/null 2>&1; then
  if [[ -n "${COSIGN_PRIVATE_KEY:-}" ]]; then
    cosign sign-blob \
      --key env://COSIGN_PRIVATE_KEY \
      --bundle "$bundle_file" \
      --yes \
      "$artifact"
    printf '\nCosign bundle: `%s`\n' "$(basename "$bundle_file")" >> "${out_dir}/README.md"
  else
    printf '\nCosign was installed, but COSIGN_PRIVATE_KEY was not set; checksum evidence only.\n' >> "${out_dir}/README.md"
  fi
else
  printf '\nCosign was not installed; checksum evidence only.\n' >> "${out_dir}/README.md"
fi
