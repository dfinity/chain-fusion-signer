use crate::utils::pic_canister::PicCanisterTrait;
use candid::{encode_one, CandidType, Principal};
use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;
use ic_chain_fusion_signer_api::types::{Arg, InitArg};
use pocket_ic::{CallError, PocketIc, PocketIcBuilder};
use std::{
    env,
    fs::read,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use super::mock::CONTROLLER;

fn workspace_dir() -> PathBuf {
    let output = std::process::Command::new(env!("CARGO"))
        .arg("locate-project")
        .arg("--workspace")
        .arg("--message-format=plain")
        .output()
        .unwrap()
        .stdout;
    let cargo_path = Path::new(std::str::from_utf8(&output).unwrap().trim());
    cargo_path.parent().unwrap().to_path_buf()
}

// Default file paths relative to the cargo workspace.
pub const BITCOIN_CANISTER_ID: &str = "g4xu7-jiaaa-aaaan-aaaaq-cai";

// This is necessary to deploy the bitcoin canister.
// This is a struct based on the `InitConfig` from the Bitcoin canister.
// Reference: https://github.com/dfinity/bitcoin-canister/blob/52c160168c478d5bce34b7dc5bacb78243c9d8aa/interface/src/lib.rs#L553
//
// The only field that matters is `network`. The others are considered dummy and set to `None` anyway.
#[derive(CandidType)]
struct BitcoinInitConfig {
    stability_threshold: Option<u64>,
    network: Option<BitcoinNetwork>,
    blocks_source: Option<String>,
    syncing: Option<String>,
    fees: Option<String>,
    api_access: Option<String>,
    disable_api_if_not_fully_synced: Option<String>,
    watchdog_canister: Option<Principal>,
    burn_cycles: Option<String>,
    lazily_evaluate_fee_percentiles: Option<String>,
}
