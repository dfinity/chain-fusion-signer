#!/usr/bin/env bash
CANISTER=bitcoin
CANDID="$(jq -r ".canisters.$CANISTER.candid" dfx.json)"

if [[ "$CANDID" == http* ]]; then
  TMPFILE="$(mktemp)"
  trap 'rm -f "$TMPFILE"' EXIT
  curl -sSfL "$CANDID" -o "$TMPFILE"
  CANDID="$TMPFILE"
fi

didc bind -t rs "$CANDID" --config "scripts/bind/pic/$CANISTER.toml" >"src/signer/canister/tests/it/canister/$CANISTER.rs"
