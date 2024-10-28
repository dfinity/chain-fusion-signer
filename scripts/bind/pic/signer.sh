#!/usr/bin/env bash
CANISTER=signer
didc bind -t rs "$(jq -r .canisters.$CANISTER.candid dfx.json)" --config "scripts/bind/pic/$CANISTER.toml" >"src/signer/canister/tests/it/canister/$CANISTER.rs"
