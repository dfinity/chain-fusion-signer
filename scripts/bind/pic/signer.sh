#!/usr/bin/env bash
CANISTER=signer
OUTPUT="src/signer/canister/tests/it/canister/$CANISTER.rs"
didc bind -t rs "$(jq -r .canisters.$CANISTER.candid dfx.json)" --config "scripts/bind/pic/$CANISTER.toml" >"$OUTPUT"

# didc generates `type Result = ...` which shadows std::result::Result,
# breaking the `Result<T, String>` return types in the generated impl block.
sed -e 's/^pub(crate) type Result = /pub(crate) type Result_ = /' \
  -e 's/Result<Result,/Result<Result_,/' \
  "$OUTPUT" >"$OUTPUT.tmp" && mv "$OUTPUT.tmp" "$OUTPUT"
