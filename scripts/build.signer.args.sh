#!/usr/bin/env bash
set -euo pipefail

print_help() {
  cat <<-EOF
	Creates the Chain Fusion Signer installation arguments:

	- The installation args are computed based on the target network,
	      determined by the DFX_NETWORK environment variable.

	The file is installed at the locations defined for 'signer' in 'dfx.json'.
	EOF
}

[[ "${1:-}" != "--help" ]] || {
  print_help
  exit 0
}

DFX_NETWORK="${DFX_NETWORK:-local}"

ARG_FILE="$(jq -r .canisters.signer.init_arg_file dfx.json)"

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
         cycles_ledger = opt principal "$CANISTER_ID_CYCLES_LEDGER";
     }
  })
EOF

# ... Also create the binary file, for use in proposals
ARG_BIN="${ARG_FILE%.did}.bin"
didc encode "$(cat "$ARG_FILE")" | xxd -r -p >"$ARG_BIN"

####
# Success
cat <<EOF
SUCCESS: The signer argument file has been created:
signer install args as candid: $(sha256sum "$ARG_FILE")
signer install args as binary: $(sha256sum "$ARG_BIN")
EOF
