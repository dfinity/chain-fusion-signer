# Hacking

This document lists information useful for development and deployment purpose.

For a concise, end-to-end release checklist (version bump, staging, and the production NNS proposal), see [RELEASE.md](RELEASE.md).

## Table of content

- [Release checklist](RELEASE.md)
- [Demo](#demo)
- [Local tooling](#local-tooling)

## Demo

This repository contains the chain fusion signer itself, and also an example application with front end and back end canisters.

To deploy the demo:

```
dfx start --clean --background
dfx deploy
```

You can now visit the front end in your browser and sign messages!

## Local tooling

Most development tools are listed in `dev-tools.json` and can be installed with `./scripts/setup <tool>`. A few need extra steps, especially on macOS.

### ic-admin

`./scripts/setup ic-admin` downloads a **Linux** binary. On macOS, fetch the `darwin` build of the pinned version (the `ic-admin` version in `dev-tools.json`) into `~/.local/bin` instead, and ensure `~/.local/bin` is on your `PATH`. From the repo root:

```
mkdir -p ~/.local/bin
VERSION="$(jq -r '."ic-admin".version' dev-tools.json)"
curl -Lf "https://github.com/dfinity/ic/releases/download/$VERSION/ic-admin-x86_64-darwin.gz" | gunzip > ~/.local/bin/ic-admin
chmod 755 ~/.local/bin/ic-admin
```

This is an x86_64 binary; on Apple Silicon it runs under Rosetta 2.

### Docker

Install with `brew install --cask docker-desktop`, then launch Docker.app once to start the daemon (the cask's final CLI symlink step needs `sudo`). On Apple Silicon, reproducible builds (`./scripts/docker-build`) run under `linux/amd64` emulation, so they are slower. A daemon-only alternative is Colima (`brew install docker colima && colima start`).

### bash >= 5

Some scripts (e.g. `./scripts/docker-build`) require bash >= 5; macOS ships bash 3.2. Install a newer one with `brew install bash`.
