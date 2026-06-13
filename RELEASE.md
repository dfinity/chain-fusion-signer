# Release checklist

A concise, end-to-end checklist for cutting a new release, deploying to `staging`, and upgrading production via an NNS proposal.

For a throwaway **test release**, just push any tag and the [Release](.github/workflows/release.yml) workflow builds the artifacts and creates a release for that tag. The sections below describe a full **production release**.

## 1. Cut the release

- Trigger the [Version Bump and Release Branch Creation](.github/workflows/bump-version.yml) workflow from the GitHub Actions tab, choosing the bump type (`patch | minor | major | alpha | beta | rc`).
  - This runs `./scripts/version-bump`, which bumps `[workspace.package].version` in the root `Cargo.toml` and regenerates `Cargo.lock`, then opens a release PR from a `release/v*` branch.
- Review and merge the release PR.

## 2. Tag, build, and publish

On merge, the following chain runs automatically:

- [Tag on Merge from Release Branch](.github/workflows/tag-release.yml) tags the merge commit with `v<version>`.
- [Release](.github/workflows/release.yml) builds the artifacts and creates a **draft** GitHub release.
- [Deploy to Staging](.github/workflows/deploy-staging.yml) deploys the release artifacts to `staging` (see step 3).

Then, by hand:

- Sanity check the draft release artifacts (the `report.txt` hashes and `commit.txt`).
- Write the release notes.
- Publish the release: `gh release edit v<version> --draft=false`.

### By hand

Sections 1 and 2 can also be done by hand:

- Ensure the development tools in `dev-tools.json` are installed; in particular you may need `./scripts/setup cargo-edit`.
- Create a release branch.
- Bump the version with `./scripts/version-bump [patch | minor | major | alpha | beta | rc]` (default: `patch`), then merge the release branch.
- Tag the merged commit with `./scripts/release`. This creates and pushes the tag; the Release GitHub action then builds the artifacts and creates the draft release.
- Sanity check the artifacts, write the release notes, and publish the release (as above).

## 3. Deploy to `staging`

- This happens **automatically** when the Release workflow completes for a `v*` tag.
- Because the release artifacts are built for the `ic` network, the workflow passes an explicit `Upgrade` argument so the existing staging configuration (e.g. `ecdsa_key_name = "test_key_1"`) is preserved instead of installing the `ic` init args.
- The canister is upgraded with the `DFX_DEPLOY_KEY_STAGING` identity, which must be a controller of the staging canister.
- Verify the upgrade:
  - `dfx canister info signer --network staging` — the module hash matches the release Wasm hash, and `git:tags` / `git:commit` metadata are correct.
  - `dfx canister call signer config --network staging` — the config still shows the staging values.
- To deploy a different ref or rebuild from scratch, trigger the Deploy to Staging workflow manually from the GitHub Actions tab (it makes a fresh reproducible docker build).

### By hand

If you are a controller of the staging canister, you can deploy directly.

A quick deploy (builds locally for the `staging` network, so it uses the correct staging args):

```
dfx deploy signer --network staging
```

Or deploy a reproducible docker build:

```
# Reproducible build; artifacts are placed in ./out/ (same location as `dfx build signer --ic`).
./scripts/docker-build

# Inspect the Wasm and install arguments in ./out/, then deploy:
dfx canister install signer --mode upgrade --upgrade-unchanged --argument '(null)' --network staging
```

Note: the docker build produces `ic` install args, so pass `--argument '(null)'` to `dfx canister install` to preserve the existing staging configuration (an `Upgrade` argument), the same way the automated workflow does — installing the `ic` args would otherwise overwrite the staging key.

If you are not a controller, you may request a canister upgrade via Orbit; contact Leon Tan for the latest Orbit deployment instructions.

## 4. Upgrade production (NNS proposal)

- Ensure the GitHub release for the tag has been **published** (step 2).
- Trigger the [Prepare Production Proposal](.github/workflows/prepare-proposal.yml) workflow with the release tag. It downloads the release assets, runs a reproducible docker build, verifies the Wasm and argument hashes match the release, generates `release/PROPOSAL.md` and `release/ROLLBACK.md`, and uploads everything as the `proposal-$TAG` workflow artifact.
- On your machine (see prerequisites below):
  - Check out the release commit.
  - Delete any old `release/` directory and download the `proposal-$TAG` artifact into `release/`.
  - Install `ic-admin`: `./scripts/setup ic-admin`.
  - Review `release/PROPOSAL.md` and `release/ROLLBACK.md`.
  - Run `./scripts/propose`, review the printed `ic-admin` command very carefully, then submit.
- Schedule an appointment with trusted neurons to vote on the proposal.

### Local prerequisites for submitting the proposal

`./scripts/propose` runs `ic-admin` and signs the NNS submission with an HSM. Before running it, make sure:

- **`ic-admin`** is installed for your platform. The standard `./scripts/setup ic-admin` downloads a **Linux** binary, so on macOS fetch the `darwin` build of the pinned version (the `ic-admin` version in `dev-tools.json`) into `~/.local/bin` instead, and ensure `~/.local/bin` is on your `PATH`. From the repo root:

  ```
  mkdir -p ~/.local/bin
  VERSION="$(jq -r '."ic-admin".version' dev-tools.json)"
  curl -Lf "https://github.com/dfinity/ic/releases/download/$VERSION/ic-admin-x86_64-darwin.gz" | gunzip > ~/.local/bin/ic-admin
  chmod 755 ~/.local/bin/ic-admin
  ```

  This is an x86_64 binary; on Apple Silicon it runs under Rosetta 2.

- The **HSM is connected** and you have its PIN. For the `ic` network, `propose` authenticates with `--use-hsm --key-id 01 --slot 0` and prompts for the PIN.
- Your **NNS neuron ID** is ready. `propose` reads it from `~/.config/dfx/prod-neuron` if present, otherwise it prompts. The HSM identity must be a controller or hotkey of that neuron.
- **`gh`** is authenticated (used to download release assets) and **`dfx`** is installed.

### Running the proposal scripts by hand

The proposal scripts can also be run directly instead of via the Prepare Production Proposal workflow: `./scripts/proposal-assets -t v<version>`, then verify with a local `./scripts/docker-build`, then `./scripts/proposal-template -t v<version>`, then `./scripts/propose`. The manual fallback additionally requires **Docker** and **bash >= 5**, since `./scripts/docker-build` needs both:

- **Docker**: install with `brew install --cask docker-desktop`, then launch Docker.app once to start the daemon (the cask's final CLI symlink step needs `sudo`). On Apple Silicon, the reproducible build runs under `linux/amd64` emulation, so it is slower. A daemon-only alternative is Colima (`brew install docker colima && colima start`).
- **bash >= 5**: macOS ships bash 3.2; install a newer one with `brew install bash`.

> Note: run the proposal scripts from an up-to-date `main` checkout rather than the release tag, so you pick up the latest tooling fixes (the scripts target the release via `-t v<version>` and do not need to be run from the release commit).
