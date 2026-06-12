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

For a production release, the flow is automated by chained GitHub actions:

- Trigger the [Version Bump and Release Branch Creation](.github/workflows/bump-version.yml) workflow from the GitHub Actions tab, choosing the bump type (`patch|minor|major|alpha|beta|rc`).
  - This runs `./scripts/version-bump` and opens a release PR from a `release/v*` branch.
- Review and merge the release PR.
- On merge, the [Tag on Merge from Release Branch](.github/workflows/tag-release.yml) workflow tags the merge commit with `v<version>`, which in turn triggers:
  - The [Release](.github/workflows/release.yml) workflow, which builds the artifacts and creates a draft GitHub release.
  - On completion of the Release workflow, the [Deploy to Staging](.github/workflows/deploy-staging.yml) workflow, which deploys the release artifacts to `staging` (see below).
- Sanity check the release artifacts.
- Write some release notes for the GitHub release, if you wish.
- Make the release public.

### Manual release

The same can be done by hand:

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

Merging a release PR deploys the new version to `staging` automatically, via the [Deploy to Staging](.github/workflows/deploy-staging.yml) workflow. It runs when the [Release](.github/workflows/release.yml) workflow completes successfully for a `v*` tag and deploys exactly the Wasm built by that release. Note that the release artifacts are built for the `ic` network, so the workflow passes an explicit `Upgrade` argument to preserve the existing staging configuration instead of installing the `ic` init args. The workflow can also be triggered manually from the GitHub Actions tab to deploy any ref, in which case it makes a fresh reproducible docker build. Either way, the canister is upgraded with the `DFX_DEPLOY_KEY_STAGING` identity, which must be a controller of the staging canister.

Alternatively, by hand: if you are a controller of the staging canister, a quick release can be made with:

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
- Trigger the [Prepare Production Proposal](.github/workflows/prepare-proposal.yml) workflow from the GitHub Actions tab with the release tag. It will:
  - Download the release Wasm and arguments to `release/ci`.
  - Run a reproducible docker build and verify that the Wasm and argument file hashes match the release assets.
  - Create `release/PROPOSAL.md` and `release/ROLLBACK.md` and upload them, together with the release assets, as a workflow artifact.
- Check out the release commit.
- Delete any old release directory and download the `proposal-$TAG` workflow artifact into `release/`.
- Install the corresponding `ic-admin`: `./scripts/setup ic-admin`
- Run: `./scripts/propose`
  - Verify the proposal very carefully, then submit the proposal.
- Create an appointment with trusted neurons to vote on the proposal.

The same can be done by hand:

- Check out the release commit
- Delete any old release directory.
- Install the corresponding `ic-admin`: `./scripts/setup ic-admin`
- Run: `./scripts/proposal-assets -t $TAG`
  - Verify that `release/ci` contains the release Wasm and arguments.
  - Run a docker build locally and verify that the Wasm and argument file hashes match.
- Run: `./scripts/proposal-template -t $TAG`
  - Verify that `release/PROPOSAL.md` has been created.
- Run: `./scripts/propose`
  - Verify the proposal very carefully, then submit the proposal.
- Create an appointment with trusted neurons to vote on the proposal.
