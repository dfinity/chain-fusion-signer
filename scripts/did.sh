#!/usr/bin/env bash

[[ "${1:-}" != "--help" ]] || {
  cat <<-EOF
	Generates the candid file for the signer
	EOF

  exit 0
}

function generate_did() {
  local canister candid_file
  canister=$1
  candid_file="src/$canister/canister/$canister.did"

  test -e "target/wasm32-unknown-unknown/release/$canister.wasm" ||
    cargo build \
      --target wasm32-unknown-unknown \
      --release --package "$canister"

  # cargo install candid-extractor
  candid-extractor "target/wasm32-unknown-unknown/release/${canister//-/_}.wasm" >"$candid_file"
}

CANISTERS=signer

for canister in ${CANISTERS//,/ }; do
  generate_did "$canister"
done
