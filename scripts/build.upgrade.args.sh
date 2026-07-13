#!/usr/bin/env bash
set -euo pipefail

print_help() {
  cat <<-EOF
	Creates the Chain Fusion Signer **upgrade** argument: '(variant { Upgrade })'.

	Unlike the Init args (see build.signer.args.sh), this argument is
	network-independent and tells the canister's post_upgrade hook to keep the
	existing configuration. Use it for every upgrade proposal; the Init args are
	only for the initial installation, and submitting them on an upgrade would
	overwrite the whole canister config.

	Usage: build.upgrade.args.sh [OUTPUT_DID_PATH]
	  OUTPUT_DID_PATH defaults to out/signer.upgrade.args.did
	  The matching .hex and .bin files are written alongside it.
	EOF
}

[[ "${1:-}" != "--help" ]] || {
  print_help
  exit 0
}

ARG_DID="${1:-out/signer.upgrade.args.did}"
ARG_HEX="${ARG_DID%.did}.hex"
ARG_BIN="${ARG_DID%.did}.bin"

mkdir -p "$(dirname "$ARG_DID")"
echo '(variant { Upgrade })' >"$ARG_DID"
didc encode "$(cat "$ARG_DID")" | tee "$ARG_HEX" | xxd -r -p >"$ARG_BIN"

cat <<EOF
SUCCESS: The signer upgrade argument file has been created:
signer upgrade args as candid: $(sha256sum "$ARG_DID")
signer upgrade args as binary: $(sha256sum "$ARG_BIN")
EOF
