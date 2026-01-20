#!/bin/bash

POCKET_IC_SERVER_VERSION=9.0.2
UPGRADE_VERSIONS="v0.0.13,v0.0.19,v0.0.25"
BITCOIN_CANISTER_RELEASE="2024-08-30"
BITCON_CANISTER_WASM="ic-btc-canister.wasm.gz"
SIGNER_CANISTER_WASM="signer.wasm"

if [ -f "./$SIGNER_CANISTER_WASM" ]; then
  echo "Use existing $SIGNER_CANISTER_WASM canister."
  # If a signer wasm file exists at the root, it will be used for the tests.
  export SIGNER_CANISTER_WASM_FILE="/$SIGNER_CANISTER_WASM"
else
  echo "Building signer canister."
  cargo build --locked --target wasm32-unknown-unknown --release -p signer
  # Otherwise, the new wasm file will be used.
  export SIGNER_CANISTER_WASM_FILE="/target/wasm32-unknown-unknown/release/$SIGNER_CANISTER_WASM"
fi

if [ -f "./$BITCON_CANISTER_WASM" ]; then
  echo "Use existing $BITCON_CANISTER_WASM canister."
else
  echo "Downloading bitcoin_canister canister."
  curl -sSL "https://github.com/dfinity/bitcoin-canister/releases/download/release%2F$BITCOIN_CANISTER_RELEASE/ic-btc-canister.wasm.gz" -o $BITCON_CANISTER_WASM
fi
# Setting the environment variable that will be used in the test to load that particular file relative to the cargo workspace.
export BITCOIN_CANISTER_WASM_FILE="/$BITCON_CANISTER_WASM"

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

export POCKET_IC_BIN="${PWD}/${POCKET_IC_SERVER_PATH}"
export POCKET_IC_MUTE_SERVER=""

# Run tests

echo "Running signer integration tests."
cargo test -p signer "${@}"
