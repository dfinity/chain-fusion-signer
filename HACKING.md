# Hacking

This document lists information useful for development purpose.

For a concise, end-to-end release checklist (version bump, staging, and the production NNS proposal), see [RELEASE.md](RELEASE.md).

## Table of content

- [Release checklist](RELEASE.md)
- [Local deployment](#local-deployment)
- [Tests](#tests)
- [Local tooling](#local-tooling)

## Local deployment

The project is configured for [icp-cli](https://github.com/dfinity/icp-cli) in `icp.yaml`. Install the pinned version with `./scripts/setup icp-cli`, then deploy the signer, the cycles depositor and an example backend canister to a local network:

```
icp network start -d
icp deploy
```

The local network runs the cycles ledger at its mainnet ID, `um5iw-rqaaa-aaaaq-qaaba-cai`. Mainnet canister IDs are in `.icp/data/mappings/<environment>.ids.json`. Stop the network with `icp network stop`.

## Tests

- `./scripts/test-rs` runs the unit tests.
- `./scripts/test.signer.sh` runs the PocketIC integration tests. It downloads the third-party canisters pinned in `scripts/test-canisters.json`.

## Local tooling

Most development tools are listed in `dev-tools.json` and can be installed with `./scripts/setup <tool>`. A few need extra steps, especially on macOS.

### ic-admin and didc

`./scripts/setup ic-admin didc` installs both into `~/.local/bin`, which must be on your `PATH`. On macOS it installs the macOS builds; they are x86_64 binaries, so on Apple Silicon they run under Rosetta 2.

### Docker

Install with `brew install --cask docker-desktop`, then launch Docker.app once to start the daemon (the cask's final CLI symlink step needs `sudo`). On Apple Silicon, reproducible builds (`./scripts/docker-build`) run under `linux/amd64` emulation, so they are slower. A daemon-only alternative is Colima (`brew install docker colima && colima start`).

### bash >= 5

Some scripts (e.g. `./scripts/docker-build`) require bash >= 5; macOS ships bash 3.2. Install a newer one with `brew install bash`.
