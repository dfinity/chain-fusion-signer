#!/usr/bin/env bash

function generate_did() {
  local canister=$1
  canister_root="src/$canister"

  test -e "target/wasm32-unknown-unknown/release/$canister.wasm" ||
    cargo build --manifest-path="$canister_root/Cargo.toml" \
      --target wasm32-unknown-unknown \
      --release --package "$canister"

  # cargo install candid-extractor
  candid-extractor "target/wasm32-unknown-unknown/release/${canister//-/_}.wasm" >"$canister_root/$canister.did"
}

CANISTERS=signer

for canister in ${CANISTERS//,/ }; do
  generate_did "$canister"
  dfx generate "$canister"
done
