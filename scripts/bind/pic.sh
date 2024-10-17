#!/usr/bin/env bash
set -euo pipefail

[[ "${1:-}" != "--help" ]] || {
  cat <<-EOF
	Generates candid file and javascript bindings for the signer
	EOF

  exit 0
}

mapfile -t canisters < <(jq -r '.canisters|keys|.[]' dfx.json)
for canister in "${canisters[@]}"; do
  pic_binding_builder="./scripts/bind/pic/${canister}.sh"
  if test -f "$pic_binding_builder"; then
    echo "INFO: Creating pic bindings for $canister..."
    bash -x "$pic_binding_builder"
  else
    echo "INFO: No pic binding script for $canister at $pic_binding_builder"
  fi
done
