#!/usr/bin/env bash
CANISTER=cycles_depositor
CANDID="$(jq -r ".canisters.$CANISTER.candid" dfx.json)"

if [[ "$CANDID" == http* ]]; then
  TMPFILE="$(mktemp)"
  trap 'rm -f "$TMPFILE"' EXIT
  curl -sSfL "$CANDID" -o "$TMPFILE"
  CANDID="$TMPFILE"
elif [[ ! -f "$CANDID" ]]; then
  LEDGER_RELEASE="v1.0.1"
  CANDID_URL="https://github.com/dfinity/cycles-ledger/releases/download/cycles-ledger-${LEDGER_RELEASE}/depositor.did"
  TMPFILE="$(mktemp)"
  trap 'rm -f "$TMPFILE"' EXIT
  curl -sSfL "$CANDID_URL" -o "$TMPFILE"
  CANDID="$TMPFILE"
fi

didc bind -t rs "$CANDID" --config "scripts/bind/pic/$CANISTER.toml" >"src/signer/canister/tests/it/canister/$CANISTER.rs"
