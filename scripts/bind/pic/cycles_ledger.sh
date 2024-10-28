#!/usr/bin/env bash
didc bind -t rs .dfx/local/canisters/cycles_ledger/cycles_ledger.did --config scripts/bind/pic/cycles_ledger.toml >src/signer/canister/tests/it/canister/cycles_ledger.rs
