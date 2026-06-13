# Hacking

This document lists information useful for development and deployment purpose.

For a concise, end-to-end release checklist (version bump, staging, and the production NNS proposal), see [RELEASE.md](RELEASE.md).

## Table of content

- [Release checklist](RELEASE.md)
- [Demo](#demo)

## Demo

This repository contains the chain fusion signer itself, and also an example application with front end and back end canisters.

To deploy the demo:

```
dfx start --clean --background
dfx deploy
```

You can now visit the front end in your browser and sign messages!
