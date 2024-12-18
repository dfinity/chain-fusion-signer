#!/usr/bin/env bash
set -euo pipefail

print_help() {
  cat <<-EOF
	Creates the Chain Fusion Signer installation files:

	- The Wasm and Candid files are built.
	- The installation args are computed based on the target network,
	      determined by the DFX_NETWORK environment variable.

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
ARG_FILE="$(jq -r .canisters.signer.init_arg_file dfx.json)"
BUILD_DIR="target/wasm32-unknown-unknown/release"
COMMIT_FILE="target/commit"
TAGS_FILE="target/tags"

####
# Gets commit and tag information, if available.
mkdir -p target
if test -d .git; then
  scripts/commit-metadata
else
  touch "$COMMIT_FILE" "$TAGS_FILE"
fi
# Keep just the tags with semantic versions
grep -E '^v[0-9]' "$TAGS_FILE" >"${TAGS_FILE}.semver" || true # No match is fine.

####
# Builds the Wasm without metadata
cargo build --locked --target wasm32-unknown-unknown --release -p signer

####
# Builds the candid file
mkdir -p "$(dirname "$CANDID_FILE")"
candid-extractor "$BUILD_DIR/signer.wasm" >"$CANDID_FILE"

####
# Optimize Wasm and set metadata
ic-wasm \
  "$BUILD_DIR/signer.wasm" \
  -o "$BUILD_DIR/signer.optimized.wasm" \
  shrink

# adds the content of $canister.did to the `icp:public candid:service` custom section of the public metadata in the wasm
ic-wasm "$BUILD_DIR/signer.optimized.wasm" -o "$BUILD_DIR/signer.candid.wasm" metadata candid:service -f "$CANDID_FILE" -v public
ic-wasm "$BUILD_DIR/signer.candid.wasm" -o "$BUILD_DIR/signer.commit.wasm" metadata git_commit_id -f "$COMMIT_FILE" -v public
ic-wasm "$BUILD_DIR/signer.commit.wasm" -o "$BUILD_DIR/signer.metadata.wasm" metadata git_tags -f "${TAGS_FILE}.semver" -v public

gzip -fn "$BUILD_DIR/signer.metadata.wasm"

mkdir -p "$(dirname "$WASM_FILE")"
mv "$BUILD_DIR/signer.metadata.wasm.gz" "$WASM_FILE"

####
# Computes the install args, overwriting any existing args file.
./scripts/build.signer.args.sh

####
# Adds the candid file to the output directory
cp src/signer/canister/signer.did out/

####
# Success
scripts/build.signer.report.sh
