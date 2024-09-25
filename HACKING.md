# Hacking

This document lists information useful for development and deployment purpose.

## Table of content

- [Demo](#demo)
- [GitHub Release](#github-release)
- [Deploy to `staging`](#deploy-to-staging)

## Demo

This repository contains the chain fusion signer itself, and also an example application with front end and back end canisters.

To deploy the demo:

```
dfx start --clean --background
dfx deploy
```

You can now visit the frontend in your browser and sign messages!

## GitHub Release

Releases are created by a GitHub action.

For a test release, just push any tag and a release will be created for that tag.

For a production release:

- Create a release branch.
- Update the version numbers in the repo, with: `./scripts/version-bump`
- Merge the release branch.
- Tag the merged code with: `scripts/release`.
  - Note: This will create a tag and push it to GitHub. A GitHub action will then create a release.

## Deploy to `staging`

If you are a controller of the staging canister, a quick release can be made with:

```
dfx deploy signer --network staging
```

If you are acontroller and wish to deploy a reproducible docker build:

```
./scripts/docker-build
dfx canister install signer --mode upgrade --upgrade-unchanged --network staging
```

If you are not a controller, you may request a canister upgrade via Orbit. Please contact Leon Tan for the latest Orbit deployment instructions.
