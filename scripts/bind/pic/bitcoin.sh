#!/usr/bin/env bash
CANISTER=bitcoin
didc bind -t rs ".dfx/local/canisters/$CANISTER/$CANISTER.did" --config "scripts/bind/pic/$CANISTER.toml" >"src/signer/canister/tests/it/canister/$CANISTER.rs"