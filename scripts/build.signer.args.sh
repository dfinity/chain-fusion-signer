#!/usr/bin/env bash
set -euo pipefail

print_help() {
  cat <<-EOF
	Creates the Chain Fusion Signer installation arguments:

	- The installation args are computed based on the target network,
	      determined by the DFX_NETWORK environment variable.

	The file is installed at the locations defined for 'signer' in 'dfx.json' and 'icp.yaml'.

	Environment variables:
	- DFX_NETWORK: The target network: local (default), staging, beta or ic.
	- CANISTER_ID_CYCLES_LEDGER: The cycles ledger canister ID; defaults to the mainnet ID,
	      which is also the ID used on local networks.
	EOF
}

[[ "${1:-}" != "--help" ]] || {
  print_help
  exit 0
}

DFX_NETWORK="${DFX_NETWORK:-local}"
CANISTER_ID_CYCLES_LEDGER="${CANISTER_ID_CYCLES_LEDGER:-um5iw-rqaaa-aaaaq-qaaba-cai}"

# Note: This path must match the 'signer' entries in 'dfx.json' and 'icp.yaml'.
ARG_FILE="out/signer.args.did"

####
# Computes the install args, overwriting any existing args file.

# .. Computes fields for the init args.
# The root key is null on every network: on mainnet the signer uses its hardcoded mainnet root
# key, and elsewhere the signer only stores the key without using it.
ic_root_key_der="null"
case "$DFX_NETWORK" in
"staging")
  ECDSA_KEY_NAME="test_key_1"
  ;;
"ic" | "beta")
  ECDSA_KEY_NAME="key_1"
  ;;
*)
  ECDSA_KEY_NAME="dfx_test_key"
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
ARG_HEX="${ARG_FILE%.did}.hex"
ARG_BIN="${ARG_FILE%.did}.bin"
didc encode "$(cat "$ARG_FILE")" | tee "$ARG_HEX" | xxd -r -p >"$ARG_BIN"

####
# Success
cat <<EOF
SUCCESS: The signer argument file has been created:
signer install args as candid: $(sha256sum "$ARG_FILE")
signer install args as binary: $(sha256sum "$ARG_BIN")
EOF
