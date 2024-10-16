#!/usr/bin/env bash
didc bind -t rs "$(jq -r .canisters.cycles_depositor.candid dfx.json)" --config scripts/bind/pic/cycles_depositor.toml >src/signer/canister/tests/it/canister/cycles_depositor.rs
