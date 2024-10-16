#!/usr/bin/env bash
set -euo pipefail
canisters=($(jq -r '.canisters|keys|.[]' dfx.json))
for canister in "${canisters[@]}"; do
  pic_binding_builder="./scripts/bind/pic/${canister}.sh"
  if test -f "$pic_binding_builder"; then
    echo "INFO: Creating pic bindings for $canister..."
    bash -x "$pic_binding_builder"
  else
    echo "INFO: No pic binding script for $canister at $pic_binding_builder"
  fi
done
