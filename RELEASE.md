# Release runbook

An end-to-end, step-by-step guide for cutting a release, deploying it to `staging`, and upgrading production via an NNS proposal.

It is written as a **by-hand runbook**, because the CI automation is not fully provisioned yet (see [Automation](#automation-once-secrets-are-provisioned) at the end). Follow the numbered steps in order; each ends with a **CHECK** you should confirm before moving on.

> **Throwaway test release:** if you only need build artifacts, push any tag and the [Release](.github/workflows/release.yml) workflow builds them and creates a release for that tag. The steps below are for a full **production release**.

Canisters: `signer` is `tdxud-2yaaa-aaaad-aadiq-cai` on `staging` and `grghe-syaaa-aaaar-qabyq-cai` on `ic` (production).

## Preflight — have these ready before you start

- **GitHub:** `gh` authenticated; you can open and merge PRs, and a second person can approve (you cannot approve your own PR).
- **Build tooling:** `cargo-edit` (`./scripts/setup cargo-edit`), Docker, and `bash >= 5` (for the reproducible `./scripts/docker-build`).
- **Staging deploy (step 5):** a `dfx` identity that is a **controller of the staging canister**.
- **Production proposal (steps 7–8):** `ic-admin` on your `PATH` (macOS needs the `darwin` build — see [HACKING.md](HACKING.md#local-tooling)), the **HSM connected with its PIN**, and your **NNS neuron ID** (the HSM identity must be a controller or hotkey of it). `propose` reads the neuron from `~/.config/dfx/prod-neuron` if present, otherwise prompts.

---

## 1. Cut the version-bump PR

1. `git fetch origin`, then create a fresh **worktree** off `origin/main` (keeps your main checkout clean) on branch `chore/release-<version>`.
2. Ensure `cargo-edit` is present: `./scripts/setup cargo-edit`.
3. Bump the version — choose the level by impact:
   - **minor** for a behaviour change (e.g. fee-semantics changes: `0.4.0` added input UTXOs, `0.5.0` added output UTXOs).
   - **patch** for fixes/maintenance only.
   ```
   ./scripts/version-bump [patch | minor | major | alpha | beta | rc]
   ```
   This bumps `[workspace.package].version` in the root `Cargo.toml` (inherited by `signer`, `ic-chain-fusion-signer-api`, `example_backend`) and regenerates `Cargo.lock`.
4. Commit just those two files: `chore(release): Bump version to <version>`.
5. Push and open a **draft** PR using the `Motivation` / `Changes` / `Tests` template.

**CHECK:** the diff is version-only (Cargo.toml + the three crate versions in Cargo.lock); CI is green; you have one approving review. Then mark the PR ready and merge it.

## 2. Tag the release

1. `git checkout main && git pull --ff-only origin main`.
2. Tag and build: `./scripts/release` — it tags the merge commit `v<version>`, pushes the tag, and watches the [Release](.github/workflows/release.yml) workflow build the artifacts.

**CHECK:** `Cargo.toml` shows `<version>` and `HEAD` is the merge commit; the Release workflow **succeeded**; a **draft** GitHub release exists with all assets (`signer.wasm.gz`, `report.txt`, `commit.txt`, `signer.did`, `signer.args.*`, `provenance.json`).

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

If you are a controller of the staging canister, deploy directly. The path below installs the **already-published, hash-verified release Wasm** (no local or docker build needed) and encodes the gotchas that otherwise turn this into back-and-forth (verified for `v0.5.0`).

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

**CHECK** `release/PROPOSAL.md`: the wasm hash matches step b; the **Candid/binary arg hashes are the production args** (`ecdsa_key_name = "key_1"`, not `test_key_1`); the target is `grghe-syaaa-aaaar-qabyq-cai`; and `release/ROLLBACK.md` points at the version currently on production.

## 8. Submit the proposal (HSM-gated)

With the HSM connected and your neuron ready:

```
./scripts/propose
```

It builds the `ic-admin` command, **prints it for review** (it authenticates with `--use-hsm --key-id 01 --slot 0` and prompts for the PIN), then signs and submits.

**CHECK:** before confirming, read the printed command carefully — target canister `grghe-syaaa-aaaar-qabyq-cai`, the wasm hash from step 3/7, and your neuron ID. After submission, confirm the proposal is listed on the NNS.

## 9. Get it voted in

Schedule an appointment with trusted neurons to vote on the proposal.

---

## Automation (once secrets are provisioned)

The repo ships a CI chain that is meant to replace the by-hand steps above, but **it has never run successfully** because its secrets are not set up. Until they are, use the runbook above. The intended flow and the missing pieces:

- **Step 1 — [Version Bump and Release Branch Creation](.github/workflows/bump-version.yml)** (opens the release PR) and **step 2 — [Tag on Merge from Release Branch](.github/workflows/tag-release.yml)** (tags on merge) authenticate as a GitHub App via the `PR_AUTOMATION_BOT_PUBLIC_APP_ID` variable and `PR_AUTOMATION_BOT_PUBLIC_PRIVATE_KEY` secret. Missing → both fail at "Create GitHub App Token" (`private-key input must be set to a non-empty string`).
- **Step 5 — [Deploy to Staging](.github/workflows/deploy-staging.yml)** runs automatically when Release completes for a `v*` tag, using the `DFX_DEPLOY_KEY_STAGING` secret (a controller identity in PEM form). Missing → fails at "Import deployment identity" (`Failed to validate pem file ... missing data`).
- **Step 7 — [Prepare Production Proposal](.github/workflows/prepare-proposal.yml)** would generate the proposal artifact, but its "Build reproducibly" step runs `git log | head -n1` under `set -o pipefail`, so `git log` gets SIGPIPE and it fails with exit code 141 (fix: `git log -1 --format=%H`).
- The **Release** build (`GITHUB_TOKEN`) and the staging deploy's payment of cycles are unaffected.

Once the three credentials are provisioned and the `prepare-proposal` step is fixed, the automated path should work; update this runbook to lead with it.
