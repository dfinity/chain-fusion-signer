didc bind -t rs "$(jq -r .canisters.cycles_depositor.candid dfx.json)" --config scripts/pic/cycles_depositor.toml > src/signer/canister/tests/it/utils/cycles_depositor.rs
