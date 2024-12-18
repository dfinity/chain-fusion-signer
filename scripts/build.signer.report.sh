#!/usr/bin/env bash
set -euo pipefail

# Hash the files:
sha256sum out/signer* >out/filelist.txt
# Get the metadata keys:
ic-wasm <(gunzip <./out/signer.wasm.gz) metadata >out/metadata_keys.txt
# Write a report
{
  printf "\nAssets:\n"
  cat out/filelist.txt
  printf "\nMetadata keys:\n"
  cat out/metadata_keys.txt
  printf "%s\n" "" "To see metadata, use ic-wasm.  For example, to see the git tags:" " ic-wasm <(gunzip < ./out/signer.wasm.gz) metadata git:tags" ""
} | tee out/report.txt
