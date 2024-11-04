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
const DEFAULT_SIGNER_WASM: &str = "/target/wasm32-unknown-unknown/release/signer.wasm";
const DEFAULT_BITCOIN_WASM: &str = "/ic-btc-canister.wasm.gz";
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

#[inline]
pub fn controller() -> Principal {
    Principal::from_text(CONTROLLER)
        .expect("Test setup error: Failed to parse controller principal")
}

impl PicSigner {
    #[allow(dead_code)]
    pub fn upgrade_latest_wasm(&self, encoded_arg: Option<Vec<u8>>) -> Result<(), String> {
        let backend_wasm_path =
            env::var("BACKEND_WASM_PATH").unwrap_or_else(|_| DEFAULT_SIGNER_WASM.to_string());

        self.upgrade_with_wasm(&backend_wasm_path, encoded_arg)
    }

    pub fn upgrade_with_wasm(
        &self,
        backend_wasm_path: &String,
        encoded_arg: Option<Vec<u8>>,
    ) -> Result<(), String> {
        let wasm_bytes = read(backend_wasm_path.clone()).expect(&format!(
            "Could not find the backend wasm: {}",
            backend_wasm_path
        ));

        let arg = encoded_arg.unwrap_or(encode_one(&Arg::Upgrade).unwrap());

        // Upgrades burn a lot of cycles.
        // If too many cycles are burnt in a short time, the canister will be throttled, so we advance time.
        // The delay here is extremely conservative and can be reduced if needed.
        self.pic.advance_time(Duration::from_secs(100_000));

        self.pic
            .upgrade_canister(
                self.canister_id,
                wasm_bytes,
                encode_one(&arg).unwrap(),
                Some(controller()),
            )
            .map_err(|e| match e {
                CallError::Reject(e) => e,
                CallError::UserError(e) => {
                    format!(
                        "Upgrade canister error. RejectionCode: {:?}, Error: {}",
                        e.code, e.description
                    )
                }
            })
    }
}

/// A test signing canister with a ic_chain_fusion_signer_api reference to the `PocketIc` instance it is installed on.
pub struct PicSigner {
    pub pic: Arc<PocketIc>,
    pub canister_id: Principal,
}
impl PicCanisterTrait for PicSigner {
    fn pic(&self) -> Arc<PocketIc> {
        self.pic.clone()
    }
    fn canister_id(&self) -> Principal {
        self.canister_id.clone()
    }
}
