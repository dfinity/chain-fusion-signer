#!/usr/bin/env bash
set -euo pipefail

print_help() {
  cat <<-EOF
	Creates the Chain Fusion Signer installation files:

	- The Wasm and Candid files are built.

	The files are installed at the locations defined for 'signer' in 'dfx.json'.
	EOF
}

[[ "${1:-}" != "--help" ]] || {
  print_help
  exit 0
}

DFX_NETWORK="${DFX_NETWORK:-local}"

CANDID_FILE="$(jq -r .canisters.signer.candid dfx.json)"
WASM_FILE="$(jq -r .canisters.signer.wasm dfx.json)"

####
# Builds the Wasm without metadata
cargo build --locked --target wasm32-unknown-unknown --release -p signer

####
# Builds the candid file
mkdir -p "$(dirname "$WASM_FILE")"
candid-extractor "target/wasm32-unknown-unknown/release/signer.wasm" >"$CANDID_FILE"

####
# Optimize Wasm and set metadata
ic-wasm \
  "target/wasm32-unknown-unknown/release/signer.wasm" \
  -o "target/wasm32-unknown-unknown/release/signer.optimized.wasm" \
  shrink

# adds the content of $canister.did to the `icp:public candid:service` custom section of the public metadata in the wasm
ic-wasm "target/wasm32-unknown-unknown/release/signer.optimized.wasm" -o "target/wasm32-unknown-unknown/release/signer.metadata.wasm" metadata candid:service -f "$CANDID_FILE" -v public

gzip -fn "target/wasm32-unknown-unknown/release/signer.metadata.wasm"

mv "target/wasm32-unknown-unknown/release/signer.metadata.wasm.gz" "$WASM_FILE"

####
# Success
cat <<EOF
SUCCESS: The signer installation files have been created:
signer candid:       $(sha256sum "$CANDID_FILE")
signer Wasm:         $(sha256sum "$WASM_FILE")
EOF
