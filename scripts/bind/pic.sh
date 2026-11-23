#!/usr/bin/env bash
set -euo pipefail

[[ "${1:-}" != "--help" ]] || {
  cat <<-EOF
	Generates canister bindings for use with pocket-ic.

	Runs every 'scripts/bind/pic/<canister>.sh'.
	EOF

  exit 0
}

for pic_binding_builder in scripts/bind/pic/*.sh; do
  echo "INFO: Creating pic bindings for $(basename "$pic_binding_builder" .sh)..."
  bash -x "$pic_binding_builder"
done
