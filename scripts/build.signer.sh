#!/usr/bin/env bash
set -euo pipefail

print_help() {
  cat <<-EOF
	Creates the Chain Fusion Signer installation files:

	- The Wasm and Candid files are built.
	- The installation args are computed based on the target network,
	      determined by the DFX_NETWORK environment variable.

	The files are installed at at the locations defined for 'signer' in 'dfx.json'.
	EOF
}

[[ "${1:-}" != "--help" ]] || {
  print_help
  exit 0
}

DFX_NETWORK="${DFX_NETWORK:-local}"

SIGNER_RELEASE="v0.1.1"
CANDID_URL="https://raw.githubusercontent.com/dfinity/chain-fusion-signer/${SIGNER_RELEASE}/src/signer/signer.did"
WASM_URL="https://github.com/dfinity/chain-fusion-signer/releases/download/${SIGNER_RELEASE}/signer.wasm.gz"

CANDID_FILE="$(jq -r .canisters.signer.candid dfx.json)"
WASM_FILE="$(jq -r .canisters.signer.wasm dfx.json)"
ARG_FILE="$(jq -r .canisters.signer.init_arg_file dfx.json)"

####
# Builds the Wasm without metadata
cargo build --locked --target wasm32-unknown-unknown --release -p signer

####
# Builds the candid file
mkdir -p "$(dirname "$WASM_FILE")"
candid-extractor "target/wasm32-unknown-unknown/release/signer.wasm" >"$CANDID_FILE"

####
# Optimize WASM and set metadata
ic-wasm \
  "target/wasm32-unknown-unknown/release/signer.wasm" \
  -o "target/wasm32-unknown-unknown/release/signer.optimized.wasm" \
  shrink

# adds the content of $canister.did to the `icp:public candid:service` custom section of the public metadata in the wasm
ic-wasm "target/wasm32-unknown-unknown/release/signer.optimized.wasm" -o "target/wasm32-unknown-unknown/release/signer.metadata.wasm" metadata candid:service -f "$CANDID_FILE" -v public

gzip -fn "target/wasm32-unknown-unknown/release/signer.metadata.wasm"

mv "target/wasm32-unknown-unknown/release/signer.metadata.wasm.gz" "$WASM_FILE"

####
# Computes the install args, overwriting any existing args file.

# .. Computes fields for the init args.
case "$DFX_NETWORK" in
"staging")
  ECDSA_KEY_NAME="test_key_1"
  # For security reasons, mainnet root key will be hardcoded in the signer canister.
  ic_root_key_der="null"
  ;;
"ic")
  ECDSA_KEY_NAME="key_1"
  # For security reasons, mainnet root key will be hardcoded in the signer canister.
  ic_root_key_der="null"
  ;;
*)
  ECDSA_KEY_NAME="dfx_test_key"
  # In order to read the root key we grab the array from the '"root_key": [...]' bit, the brackets
  # to match what candid expects ({}), replace the commas between array entries to match
  # what candid expects (semicolon) and annotate the numbers with their type (otherwise dfx assumes 'nat'
  # instead of 'nat8').
  rootkey_did=$(dfx ping "${ENV:-local}" |
    jq -r '.root_key | reduce .[] as $item ("{ "; "\(.) \($item):nat8;") + " }"')
  echo "Parsed rootkey: ${rootkey_did:0:20}..." >&2
  ic_root_key_der="opt vec $rootkey_did"
  ;;
esac

# .. Creates the init args file
rm -f "$ARG_FILE"
mkdir -p "$(dirname "$ARG_FILE")"
cat <<EOF >"$ARG_FILE"
(variant {
    Init = record {
         ecdsa_key_name = "$ECDSA_KEY_NAME";
         ic_root_key_der = $ic_root_key_der;
     }
  })
EOF

####
# Success
cat <<EOF
SUCCESS: The signer installation files have been created:
signer candid:       $(sha256sum "$CANDID_FILE")
signer Wasm:         $(sha256sum "$WASM_FILE")
signer install args: $(sha256sum "$ARG_FILE")
EOF
