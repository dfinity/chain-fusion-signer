#!/usr/bin/env bash
set -euo pipefail

print_help() {
  cat <<-EOF
	Builds the example backend canister as out/example_backend.wasm.gz.

	The Wasm is shrunk, has the candid:service metadata set and is gzipped.
	EOF
}

[[ "${1:-}" != "--help" ]] || {
  print_help
  exit 0
}

CANDID_FILE="src/example_backend/example_backend.did"
BUILD_DIR="target/wasm32-unknown-unknown/release"
WASM_FILE="out/example_backend.wasm.gz"

cargo build --locked --target wasm32-unknown-unknown --release -p example_backend
ic-wasm "$BUILD_DIR/example_backend.wasm" -o "$BUILD_DIR/example_backend.optimized.wasm" shrink
ic-wasm "$BUILD_DIR/example_backend.optimized.wasm" -o "$BUILD_DIR/example_backend.metadata.wasm" metadata candid:service -f "$CANDID_FILE" -v public
gzip -fn "$BUILD_DIR/example_backend.metadata.wasm"
mkdir -p "$(dirname "$WASM_FILE")"
mv "$BUILD_DIR/example_backend.metadata.wasm.gz" "$WASM_FILE"
echo "SUCCESS: Built $(sha256sum "$WASM_FILE")"
