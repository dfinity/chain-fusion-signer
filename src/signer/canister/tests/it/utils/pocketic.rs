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

// Default file paths relative to the cargo workspace.
pub const BITCOIN_CANISTER_ID: &str = "g4xu7-jiaaa-aaaan-aaaaq-cai";
