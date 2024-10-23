#!/usr/bin/env bash

[[ "${1:-}" != "--help" ]] || {
  cat <<-EOF
	Generates candid file and javascript bindings for the signer
	EOF

  exit 0
}

function generate_did() {
  local canister candid_file
  canister=$1
  candid_file="$(canister="$canister" jq -r '.canisters[env.canister].candid' dfx.json)"

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
  dfx generate "$canister"
done
