#!/bin/bash

POCKET_IC_SERVER_VERSION=5.0.0
UPGRADE_VERSIONS="v0.0.13,v0.0.19,v0.0.25"

# If a signer wasm file exists at the root, it will be used for the tests.

if [ -f "./signer.wasm.gz" ]; then
  # Setting the environment variable will be used in the test to load that particular file. Relative to where the test is.
  echo "Use existing signer.wasm.gz canister."
  export BACKEND_WASM_PATH="../../signer.wasm.gz"
else
  # If none exist we build the project. The test will resolve the target/wasm32-unknown-unknown/release/signer.wasm automatically as fallback if no exported BACKEND_WASM_PATH variable is set.
  echo "Building signer canister."
  cargo build --locked --target wasm32-unknown-unknown --release -p signer
fi

# We use a previous version of the release to ensure upgradability

IFS=',' read -r -a versions <<<"$UPGRADE_VERSIONS"

for version in "${versions[@]}"; do
  UPGRADE_PATH="./signer-${version}.wasm.gz"

  if [ ! -f "$UPGRADE_PATH" ]; then
    curl -sSL "https://github.com/dfinity/chain-fusion-signer/releases/download/${version}/signer.wasm.gz" -o "$UPGRADE_PATH"
  fi
done

# Download PocketIC server

POCKET_IC_SERVER_PATH="target/pocket-ic"

if [ ! -d "target" ]; then
  mkdir "target"
fi

if [[ $OSTYPE == "linux-gnu"* ]] || [[ $RUNNER_OS == "Linux" ]]; then
  PLATFORM=linux
elif [[ $OSTYPE == "darwin"* ]] || [[ $RUNNER_OS == "macOS" ]]; then
  PLATFORM=darwin
else
  echo "OS not supported: ${OSTYPE:-$RUNNER_OS}"
  exit 1
fi

if [ ! -f "$POCKET_IC_SERVER_PATH" ]; then
  echo "Downloading PocketIC."
  curl -sSL https://github.com/dfinity/pocketic/releases/download/${POCKET_IC_SERVER_VERSION}/pocket-ic-x86_64-${PLATFORM}.gz -o ${POCKET_IC_SERVER_PATH}.gz
  gunzip ${POCKET_IC_SERVER_PATH}.gz
  chmod +x ${POCKET_IC_SERVER_PATH}
else
  echo "PocketIC server already exists, skipping download."
fi

export POCKET_IC_BIN="../../${POCKET_IC_SERVER_PATH}"
export POCKET_IC_MUTE_SERVER=""

# Run tests

echo "Running signer integration tests."
cargo test -p signer "${@}"
