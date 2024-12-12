# Upgrade Chain Fusion Signer canister to v0.2.8
Commit: `e1c36f468c7b78e851332cc7dae0bfce6bc8f886`
Release: `https://github.com/dfinity/chain-fusion-signer/releases/tag/v0.2.8`
Wasm sha256 hash:     `9d76ee4646df62d1a4bddbdfb4fa78e43166c19a94595ba9e1f5f27b823c8192`
Candid argument hash: `a99d4a8355c86d2367955d833df83302b5748e6e34898e280e359f9c57541e1f`
Binary argument hash: `367a11245d7e97692843c69e9582dd8ad08eef1ff3392dfd0c944d5f1bd74a41`

The chain fusion signer is a canister that makes the internet computer threshold signing API available to web applications, off-chain clients and other-chain applications.

## Change Log

### Features
- Add support for Schnorr signatures. This is useful for cross chain applications wishing to interact with blockchains using Schnorr ed25519.
  - A useful reference may be: http://ethanfast.com/top-crypto.html where curve25519 corresponds to chains that use Schnorr ed25519.
  - We can confirm that Solana transactions can be signed with this API.  Other chains should work but have not yet been tested.

### Maintenance
- Dependencies have been updated.
- [Pocket-ic canister bindings](https://github.com/dfinity/chain-fusion-signer/tree/v0.2.8/src/signer/canister/tests/it/canister) are now generated automatically, for improved testing.
- Tooling for creating and verifying NNS proposals has been added.

## Commit Log

```
+ bash -xc "git log --format='%C(auto) %h %s' e1c36f4..e1c36f4"

```

## Wasm Verification

To build the wasm module yourself and verify its hash, run the following commands from the root of the [Chain Fusion Signer repo](https://github.com/dfinity/chain-fusion-signer):

```
git fetch  # to ensure you have the latest changes.
git checkout "e1c36f468c7b78e851332cc7dae0bfce6bc8f886"
./scripts/docker-build
```

This will generate these files:
* out/signer.wasm.gz
* out/signer.args.did
* out/signer.args.bin

