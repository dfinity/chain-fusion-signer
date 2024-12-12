# Downgrade frontend NNS Dapp canister to commit `e1c36f468c7b78e851332cc7dae0bfce6bc8f886`
Wasm sha256 hash: `9d76ee4646df62d1a4bddbdfb4fa78e43166c19a94595ba9e1f5f27b823c8192` (`release/ci/signer.wasm.gz`)

## Wasm Verification

To build the wasm module yourself and verify its hash, run the following commands from the root of the nns-dapp repo:

```
git fetch  # to ensure you have the latest changes.
git checkout "e1c36f468c7b78e851332cc7dae0bfce6bc8f886"
./scripts/docker-build
```

This will generate these files:
* out/signer.wasm.gz
* out/signer.args.did
* out/signer.args.bin

