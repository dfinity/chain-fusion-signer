#!/usr/bin/env bash

[[ "${1:-}" != "--help" ]] || {
  cat <<-EOF
	Generates candid files and bindings.
	EOF

  exit 0
}

scripts/did.sh
scripts/bind/pic.sh
