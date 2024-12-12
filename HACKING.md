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

You can now visit the front end in your browser and sign messages!

## GitHub Release

Releases are created by a GitHub action.

For a test release, just push any tag and a release will be created for that tag.

For a production release:

- Ensure that you have the development tools listed in `dev-tools.json` installed on your machine. In particular, you may need:
  - `./scripts/setup cargo-edit`
- Create a release branch.
- Update the version numbers in the git repository, with: `./scripts/version-bump [patch|minor|major|alpha|beta|rc]` (default: patch)
- Merge the release branch.
- Tag the merged code with: `scripts/release`.
  - Note: This will create a tag and push it to GitHub. A GitHub action will then create a release.
- Sanity check the release artifacts.
- Write some release notes for the GitHub release, if you wish.
- Make the release public.

## Deploy to `staging`

If you are a controller of the staging canister, a quick release can be made with:

```
dfx deploy signer --network staging
```

If you are a controller and wish to deploy a reproducible docker build:

```
# Reproducible build
./scripts/docker-build
# Note: The docker build artifacts are placed in the same location
#       as when running `dfx build signer --ic`

# Inspect the Wasm and install arguments in `./out/`.

# Deploy:
dfx canister install signer --mode upgrade --upgrade-unchanged --network staging
```

If you are not a controller, you may request a canister upgrade via Orbit. Please contact Leon Tan for the latest Orbit deployment instructions.

## Deploy to Production

- Create a GitHub release with a tag such as `v0.1.2`
  - Update the GitHub release text. It is recommended to ask the team to review the text.
  - Ensure that the release has been published.
- Check out the release commit
- Delete any old release directory.
- Run: `./scripts/proposal-assets -t $TAG`
  - Verify that `release/ci` contains the release Wasm and arguments.
  - Run a docker build locally and verify that the Wasm and argument file hashes match.
- Run: `./scripts/proposal-template -t $TAG`
  - Verify that `release/PROPOSAL.md` has been created.
- Run: `./scripts/propose`
  - Verify the proposal very carefully, then submit the proposal.
- Create an appointment with trusted neurons to vote on the proposal.
