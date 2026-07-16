# Release runbook

An end-to-end, step-by-step guide for cutting a release, deploying it to `staging`, and upgrading production via an NNS proposal.

The CI chain now drives steps 1, 2 and 5 (open the release PR, tag on merge, deploy to `staging`); its credentials are provisioned. Follow the numbered steps in order; each ends with a **CHECK** you should confirm before moving on. Steps that are still by hand — **release notes** (step 4) and the **HSM-gated production proposal** (steps 7–8) — are marked as such, and every automated step keeps its by-hand fallback under _If you need to do it manually_.

See [Automation status](#automation-status) for what has actually been exercised and the invariants that keep it working.

> **Throwaway test release:** if you only need build artifacts, push any tag and the [Release](.github/workflows/release.yml) workflow builds them and creates a release for that tag. The steps below are for a full **production release**.

Canisters: `signer` is `tdxud-2yaaa-aaaad-aadiq-cai` on `staging` and `grghe-syaaa-aaaar-qabyq-cai` on `ic` (production).

## Preflight — have these ready before you start

- **GitHub:** `gh` authenticated; you can run workflows and merge PRs, and a second person can approve (you cannot approve your own PR).
- **Build tooling** (only for the manual fallbacks and the step 7 dry run): `cargo-edit` (`./scripts/setup cargo-edit`), Docker, and `bash >= 5` (for the reproducible `./scripts/docker-build`).
- **Staging deploy (step 5):** nothing — CI deploys with `DFX_DEPLOY_KEY_STAGING`. To deploy or inspect by hand you need a `dfx` identity that is a **controller of the staging canister**.
- **Production proposal (steps 7–8):** `ic-admin` and `didc` on your `PATH` (`./scripts/setup ic-admin didc`; macOS needs the `darwin`/`macos` builds — see [HACKING.md](HACKING.md#local-tooling)), the **HSM connected with its PIN**, and your **NNS neuron ID** (the HSM identity must be a controller or hotkey of it). `propose` reads the neuron from `~/.config/dfx/prod-neuron` if present, otherwise prompts.

---

## 1. Open the release PR

Run the [Version Bump and Release Branch Creation](.github/workflows/bump-version.yml) workflow (`workflow_dispatch`) and pick the bump level by impact:

- **minor** for a behaviour change (e.g. fee-semantics changes: `0.4.0` added input UTXOs, `0.5.0` added output UTXOs).
- **patch** for fixes/maintenance only.

```
gh workflow run bump-version.yml -f version_bump=patch
```

`version_bump` accepts `patch`, `minor`, `major`, `alpha`, `beta` or `rc`.

It runs `./scripts/version-bump`, which bumps `[workspace.package].version` in the root `Cargo.toml` (inherited by `signer`, `ic-chain-fusion-signer-api`, `example_backend`) and regenerates `Cargo.lock`, then opens a PR from branch **`release/v<version>`**.

**CHECK:** the diff is version-only (Cargo.toml + the three crate versions in Cargo.lock); CI is green; you have one approving review (you cannot approve your own PR). The head branch **must** be `release/v<version>` — that is what step 2 keys off.

> **If you need to do it manually:** create a worktree off `origin/main` on branch `release/v<version>` (**not** `chore/release-<version>` — that name silently skips the tag job in step 2), run `./scripts/setup cargo-edit` then `./scripts/version-bump <level>`, commit just those two files as `chore(release): Bump version to <version>`, and open the PR.

## 2. Merge it — tag, build and staging deploy are automatic

Merging the PR from `release/v<version>` starts the whole chain:

1. [Tag on Merge from Release Branch](.github/workflows/tag-release.yml) takes the `signer` package version from `cargo metadata` and tags the merge commit `v<version>`.
2. Pushing that tag triggers [Release](.github/workflows/release.yml), which builds the artifacts and creates a **draft** GitHub release.
3. When Release completes, [Deploy to Staging](.github/workflows/deploy-staging.yml) installs those artifacts on `staging` (see step 5 for what to verify).

**CHECK:** `tag-release` **ran** rather than being skipped (it is conditional on the head branch starting with `release/v` — a skipped run means the branch was misnamed and nothing was tagged); tag `v<version>` exists on the merge commit; the Release workflow **succeeded**; a **draft** GitHub release exists with all assets (`signer.wasm.gz`, `report.txt`, `commit.txt`, `signer.did`, `signer.args.*`, `provenance.json`).

> **If you need to do it manually:** `git checkout main && git pull --ff-only origin main`, then `./scripts/release` — it tags the merge commit `v<version>`, pushes the tag, and watches the Release workflow.

## 3. Verify the draft release artifacts

```
gh release download v<version> --pattern commit.txt --pattern report.txt
```

**CHECK:** `commit.txt` equals `git rev-parse v<version>^{commit}`, and `report.txt` lists the sha256 of `out/signer.wasm.gz` and the arg files. **Note the wasm hash** — you will match it again in steps 5 and 7.

## 4. Write release notes and publish

1. Generate the auto changelog and curate it to match recent releases (a `Features` / `Fixes` / `Maintenance` summary on top, then the generated list and the `Full Changelog` compare link):
   ```
   gh api repos/dfinity/chain-fusion-signer/releases/generate-notes \
     -f tag_name=v<version> -f previous_tag_name=v<previous> -q .body
   ```
2. Set the notes: `gh release edit v<version> --notes-file <file>`.
3. Publish (public, **irreversible** — notifies watchers): `gh release edit v<version> --draft=false`.

**CHECK:** the published release shows the curated notes and is no longer a draft.

## 5. Deploy to `staging`

[Deploy to Staging](.github/workflows/deploy-staging.yml) does this for you: it runs automatically when Release completes for a `v*` tag, importing the `DFX_DEPLOY_KEY_STAGING` secret (a controller identity in PEM form) and installing the release Wasm. Normally you only **verify** it — jump to the CHECK below.

> **If you need to do it manually** (the workflow failed, or you are shipping an untagged build): if you are a controller of the staging canister, deploy directly. The path below installs the **already-published, hash-verified release Wasm** (no local or docker build needed) and encodes the gotchas that otherwise turn this into back-and-forth (verified for `v0.5.0`). For an untagged build, replace step b with `./scripts/docker-build`.

```
# a. Confirm your identity controls staging:
dfx identity get-principal                          # must appear in the Controllers list of:
dfx canister info signer --network staging

# b. Download the release Wasm into the path dfx.json expects, and verify its hash:
gh release download v<version> --pattern signer.wasm.gz --output out/signer.wasm.gz --clobber
shasum -a 256 out/signer.wasm.gz                    # must equal the report.txt hash from step 3

# c. Create the dfx workspace dir so dfx can RUN its Candid compat check instead of falling
#    back to an interactive prompt (which a non-interactive shell auto-declines):
mkdir -p .dfx/staging/canisters/signer

# d. CRITICAL — neutralize the init args. dfx.json sets `init_arg_file: out/signer.args.did`,
#    which holds the *production* args (`ecdsa_key_name = "key_1"`). dfx reads that file and
#    IGNORES a `--argument` flag, so you must overwrite it with `(null)`, or the upgrade would
#    repoint staging at the production key. `(null)` => `post_upgrade(None)` => staging config kept.
printf '(null)\n' > out/signer.args.did

# e. Install the upgrade (`--wasm` installs the prebuilt module; `--yes` confirms past the
#    benign Candid check — the interface is unchanged for a pricing-only release; for any
#    release, diff the deployed candid vs the new signer.did to be sure):
dfx canister install signer --wasm out/signer.wasm.gz \
  --mode upgrade --upgrade-unchanged --network staging --yes
```

**CHECK** (prefix queries with `DFX_WARNING=-mainnet_plaintext_identity` if a plaintext-identity confirmation appears):

- `dfx canister info signer --network staging` → **module hash equals the release wasm hash** (step 3); `git:tags` = `v<version>`, `git:commit` = the release commit.
- `dfx canister call signer config --network staging` → still `ecdsa_key_name = "test_key_1"` (**not** `key_1`).

> Alternative: rebuild from scratch with `./scripts/docker-build` and install the same way — but the published Wasm is already that reproducible artifact, so downloading it is simpler and provably identical. If you are not a controller, request the upgrade via Orbit (contact the Orbit team).

## 6. Test on staging

Exercise the real signing flow against staging via [staging.oisy.com](https://staging.oisy.com) (it is wired to the staging signer). Make a **signed transaction**; for a fee-related change, use the BTC send path specifically.

**CHECK:** the transaction signs and sends successfully end-to-end.

## 7. Prepare the production proposal (safe dry run)

Run from an up-to-date `main` checkout. These three steps only download, build, and template — **nothing touches the HSM or gets submitted**, so review everything first.

```
# a. Download the published release assets into release/ci/ (prints their sha256sums):
./scripts/proposal-assets -t v<version>

# b. Reproduce the build and verify it is bit-for-bit identical to the release:
./scripts/docker-build
sha256sum out/signer.wasm.gz                        # must equal release/ci/report.txt
#    (out/signer.args.{did,bin} must also equal the matching release/ci files)

# c. Generate the proposal text:
./scripts/proposal-template -t v<version>
#    -> release/PROPOSAL.md  (upgrade: target canister, wasm + arg hashes, changelog)
#    -> release/ROLLBACK.md  (revert to the previous version)
```

`proposal-template` runs `build.upgrade.args.sh` (which needs `didc`) to produce the upgrade argument, so `PROPOSAL.md` records the `(variant { Upgrade })` argument rather than the `Init` args (see [Init vs Upgrade argument](#init-vs-upgrade-argument) below).

**CHECK** `release/PROPOSAL.md`: the wasm hash matches step b; the **upgrade argument is `(variant { Upgrade })`** with binary arg hash `100d1390b7762eef1bc2af9d6fdd157bb800bfc3787142c435d4c28747d0ac30` (constant for every upgrade); the target is `grghe-syaaa-aaaar-qabyq-cai`; and `release/ROLLBACK.md` points at the version currently on production.

## 8. Submit the proposal (HSM-gated)

With the HSM connected and your neuron ready:

```
./scripts/propose
```

It builds the `ic-admin` command, **prints it for review** (it authenticates with `--use-hsm --key-id 01 --slot 0` and prompts for the PIN), then signs and submits.

**CHECK:** before confirming, read the printed command carefully — target canister `grghe-syaaa-aaaar-qabyq-cai`, `--mode upgrade`, the wasm hash from step 3/7, `--arg-sha256 100d1390…` (the `(variant { Upgrade })` argument), and your neuron ID. After submission, confirm the proposal is listed on the NNS.

## 9. Get it voted in

Schedule an appointment with trusted neurons to vote on the proposal.

---

## Init vs Upgrade argument

The canister's install argument is `type Arg = variant { Upgrade; Init : InitArg }`, and `post_upgrade` treats them very differently ([`src/signer/canister/src/lib.rs`](src/signer/canister/src/lib.rs)):

- `Init : record { … }` → calls `set_config`, which **replaces the entire config** with a `Config` derived from those fields.
- `Upgrade` (or no argument) → **keeps the existing config** untouched.

Therefore **upgrade proposals must use `(variant { Upgrade })`**, not the `Init` args. Submitting `Init` on an upgrade re-applies the full config and would silently overwrite any field that differs from the baked-in args (e.g. a non-default root key or cycles ledger). The `Init` args (`scripts/build.signer.args.sh`, written to `out/signer.args.*`) are only for the **initial installation**; the staging deploy (step 5) achieves the same "keep config" effect by passing `(null)`.

`scripts/propose` and `scripts/proposal-template` both call `scripts/build.upgrade.args.sh`, which encodes `(variant { Upgrade })` with `didc` — a constant, network-independent value (binary sha256 `100d1390…`). Anyone can reproduce it with `didc encode '(variant { Upgrade })'`.

---

## Automation status

The credentials are provisioned and `bump-version` is confirmed working (it opened the `v0.5.1` release PR). The rest of the chain is wired and credentialed but was first exercised by `v0.5.1` — if you are cutting an early release after that, check each stage rather than assuming.

| Workflow                                                                                             | Credential                                                                                                                               | Status                                          |
| ---------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------- |
| [bump-version](.github/workflows/bump-version.yml), [tag-release](.github/workflows/tag-release.yml) | `PR_AUTOMATION_BOT_PUBLIC_APP_ID` (org variable) + `PR_AUTOMATION_BOT_PUBLIC_PRIVATE_KEY` (org secret), via the PR-automation GitHub App | Provisioned; `bump-version` verified end-to-end |
| [deploy-staging](.github/workflows/deploy-staging.yml)                                               | `DFX_DEPLOY_KEY_STAGING` (repo secret) — a controller identity in PEM form                                                               | Provisioned; first exercised by `v0.5.1`        |
| [Release](.github/workflows/release.yml)                                                             | `GITHUB_TOKEN`                                                                                                                           | Always worked                                   |

### Invariants that keep it working

- **Never create a branch or tag named `release`.** Git stores refs as file paths, so `refs/heads/release` and `refs/heads/release/v<version>` cannot coexist — a bare `release` branch makes every `bump-version` run fail its push with `! [remote rejected] release/v<version> (directory file conflict)`. A stale `release` branch from 2024 blocked this workflow until it was deleted; the commit it pointed at is preserved by tag `v0.2.8`.
- **The release PR's head branch must be `release/v<version>`.** `tag-release` is conditional on `startsWith(head.ref, 'release/v')`; any other name (e.g. `chore/release-<version>`) makes the job **skip silently** — the PR merges, nothing is tagged, and no release is built.
- **The deploy key must stay a controller of the staging canister.** `DFX_DEPLOY_KEY_STAGING` is only useful while its principal is in the staging controller list, and the IC caps controllers at **10** — adding one to a full list fails with `The number of elements exceeds maximum allowed 10`.

### Still by hand, by design

- **Release notes** (step 4): the chain only creates a _draft_ release; a human curates the notes and publishes.
- **The production proposal** (steps 7–8): HSM-gated. [Prepare Production Proposal](.github/workflows/prepare-proposal.yml) can generate the artifact, but submission stays manual.
