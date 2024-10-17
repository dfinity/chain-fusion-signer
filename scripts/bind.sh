#!/usr/bin/env bash

[[ "${1:-}" != "--help" ]] || {
  cat <<-EOF
	Generates candid files and bindings.
	EOF

  exit 0
}

# Generate candid and bindings for the signer
scripts/did.sh
# Generate canister bindings for use with pocket-ic
scripts/bind/pic.sh
