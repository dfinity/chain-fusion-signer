use candid::{decode_one, encode_one, CandidType, Principal};
use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;
use pocket_ic::{CallError, PocketIc, PocketIcBuilder, WasmResult};
use serde::Deserialize;
use shared::types::{Arg, InitArg};
use std::fs::read;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::{env, time::Duration};

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
const BITCOIN_CANISTER_ID: &str = "g4xu7-jiaaa-aaaan-aaaaq-cai";

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

/// Backend canister installer, using the builder pattern, for use in test environmens using `PocketIC`.
///
/// # Example
/// For a default test environment:
/// ```
/// let (pic, canister_id) = BackendBuilder::default().deploy();
/// ```
/// To add a backend canister to an existing `PocketIC`:
/// ```
/// let pic = PocketIc::new();
/// let canister_id = BackendBuilder::default().deploy_to(&pic);
/// ```
/// To redeploy an existing canister:
/// ```
/// // First deployment:
/// let (pic, canister_id) = BackendBuilder::default().deploy();
/// // Subsequent deployment:
/// let canister_id = BackendBuilder::default().with_canister(canister_id).deploy_to(&pic);
/// ```
/// To customise the deployment, use the `.with_*` modifiers.  E.g.:
/// ```
/// let (pic, canister_id) = BackendBuilder::default()
///    .with_wasm("path/to/signer.wasm")
///    .with_arg(vec![1, 2, 3])
///    .with_controllers(vec![Principal::from_text("controller").unwrap()])
///    .with_cycles(1_000_000_000_000)
///    .deploy();
/// ```
#[derive(Debug)]
pub struct BackendBuilder {
    /// Canister ID of the backend canister.  If not set, a new canister will be created.
    canister_id: Option<Principal>,
    /// Cycles to add to the backend canister.
    cycles: u128,
    /// Path to the backend wasm file.
    wasm_path: String,
    /// Path to the bitcoin canister wasm file.
    bitcoin_wasm_path: String,
    /// Argument to pass to the backend canister.
    arg: Option<Arg>,
    /// Controllers of the backend canister.
    controllers: Vec<Principal>,
}
// Defaults
impl BackendBuilder {
    /// The default number of cycles to add to the backend canister on deployment.
    ///
    /// To override, please use `with_cycles()`.
    pub const DEFAULT_CYCLES: u128 = 2_000_000_000_000;
    /// The default Wasm file to deploy:
    /// - If the environment variable `SIGNER_CANISTER_WASM_FILE` is set, it will use that path.
    /// - Otherwise, it will use the `DEFAULT_SIGNER_WASM` constant.
    ///
    /// To override, please use `with_wasm()`.
    pub fn default_wasm_path() -> String {
        let workspace_dir_str = workspace_dir()
        .to_str()
        .expect("Wrong workspace directory")
        .to_owned();
    
        let wasm_name = env::var("SIGNER_CANISTER_WASM_FILE")
            .unwrap_or_else(|_| DEFAULT_SIGNER_WASM.to_string());

        workspace_dir_str + &wasm_name
    }
    /// The default Wasm file to deploy the bitcoin canister:
    /// - If the environment variable `BITCOIN_CANISTER_WASM_FILE` is set, it will use that path.
    /// - Otherwise, it will use the `DEFAULT_BITCOIN_WASM` constant.
    ///
    /// To override, please use `with_wasm()`.
    pub fn default_bitcoin_wasm_path() -> String {
        let workspace_dir_str = workspace_dir()
            .to_str()
            .expect("Wrong workspace directory")
            .to_owned();
        
        let wasm_name = env::var("BITCOIN_CANISTER_WASM_FILE")
            .unwrap_or_else(|_| DEFAULT_BITCOIN_WASM.to_string());
    
        workspace_dir_str + &wasm_name
    }
    /// The default arguments to deploy the bitcoin canister.
    pub fn default_bitcoin_arg() -> Vec<u8> {
        let init_config = BitcoinInitConfig {
            stability_threshold: None,
            network: Some(BitcoinNetwork::Regtest),
            blocks_source: None,
            syncing: None,
            fees: None,
            api_access: None,
            disable_api_if_not_fully_synced: None,
            watchdog_canister: None,
            burn_cycles: None,
            lazily_evaluate_fee_percentiles: None,
        };
        encode_one(init_config).unwrap()
    }
    /// The default argument to pass to the backend canister.
    ///
    /// To override, please use `with_arg()`.
    pub fn default_install_arg() -> Arg {
        Arg::Init(InitArg {
            ecdsa_key_name: format!("test_key_1"),
            ic_root_key_der: None,
        })
    }
    /// The default controllers of the backend canister.
    ///
    /// To override, please use `with_controllers()`.
    pub fn default_controllers() -> Vec<Principal> {
        vec![Principal::from_text(CONTROLLER)
            .expect("Test setup error: Failed to parse controller principal")]
    }
}
impl Default for BackendBuilder {
    fn default() -> Self {
        Self {
            canister_id: None,
            cycles: Self::DEFAULT_CYCLES,
            wasm_path: Self::default_wasm_path(),
            bitcoin_wasm_path: Self::default_bitcoin_wasm_path(),
            arg: None,
            controllers: Self::default_controllers(),
        }
    }
}
// Customisation
impl BackendBuilder {
    /// Sets a custom argument for the backend canister.
    #[allow(dead_code)]
    pub fn with_arg(mut self, arg: Arg) -> Self {
        self.arg = Some(arg);
        self
    }
    /// Deploys to an existing canister with the given ID.
    #[allow(dead_code)]
    pub fn with_canister(mut self, canister_id: Principal) -> Self {
        self.canister_id = Some(canister_id);
        self
    }
    /// Sets custom controllers for the backend canister.
    #[allow(dead_code)]
    pub fn with_controllers(mut self, controllers: Vec<Principal>) -> Self {
        self.controllers = controllers;
        self
    }
    /// Sets the cycles to add to the backend canister.
    #[allow(dead_code)]
    pub fn with_cycles(mut self, cycles: u128) -> Self {
        self.cycles = cycles;
        self
    }
    /// Configures the deployment to use a custom Wasm file.
    #[allow(dead_code)]
    pub fn with_wasm(mut self, wasm_path: &str) -> Self {
        self.wasm_path = wasm_path.to_string();
        self
    }
}
// Get parameters
impl BackendBuilder {
    /// Reads the backend Wasm bytes from the configured path.
    fn wasm_bytes(&self) -> Vec<u8> {
        read(self.wasm_path.clone()).expect(&format!(
            "Could not find the backend wasm: {}",
            self.wasm_path
        ))
    }

    /// Reads the bitcoin Wasm bytes from the configured path.
    fn bitcoin_wasm_bytes(&self) -> Vec<u8> {
        read(self.bitcoin_wasm_path.clone()).expect(&format!(
            "Could not find the bitcoin wasm: {}",
            self.bitcoin_wasm_path
        ))
    }
}
// Builder
impl BackendBuilder {
    /// Get or create canister.
    fn canister_id(&mut self, pic: &PocketIc) -> Principal {
        if let Some(canister_id) = self.canister_id {
            canister_id
        } else {
            let fiduciary_subnet_id = pic
                .topology()
                .get_fiduciary()
                .expect("pic should have a fiduciary subnet.");
            let canister_id = pic.create_canister_on_subnet(None, None, fiduciary_subnet_id);
            self.canister_id = Some(canister_id);
            canister_id
        }
    }
    /// Add cycles to the backend canister.
    fn add_cycles(&mut self, pic: &PocketIc) {
        if self.cycles > 0 {
            let canister_id = self.canister_id(pic);
            pic.add_cycles(canister_id, self.cycles);
        }
    }
    /// Install the backend canister.
    fn install(&mut self, pic: &PocketIc) {
        let wasm_bytes = self.wasm_bytes();
        let canister_id = self.canister_id(pic);
        let arg = self
            .arg
            .as_ref()
            .map(encode_one)
            .unwrap_or_else(|| encode_one(&Self::default_install_arg()))
            .unwrap();
        pic.install_canister(canister_id, wasm_bytes, arg, None);
    }
    /// Set controllers of the backend canister.
    fn set_controllers(&mut self, pic: &PocketIc) {
        let canister_id = self.canister_id(pic);
        pic.set_controllers(canister_id.clone(), None, self.controllers.clone())
            .expect("Test setup error: Failed to set controllers");
    }
    fn install_bitcoin(&mut self, pic: &PocketIc) {
        let canister_id =
            Principal::from_text(BITCOIN_CANISTER_ID).expect("Unexpected bitcoin canister id");
        pic.create_canister_with_id(None, None, canister_id)
            .expect("Failed creating bitcoin canister");
        let wasm_bytes = self.bitcoin_wasm_bytes();
        pic.install_canister(canister_id, wasm_bytes, Self::default_bitcoin_arg(), None);
    }
    /// Setup the backend canister.
    pub fn deploy_to(&mut self, pic: &PocketIc) -> Principal {
        let canister_id = self.canister_id(pic);
        self.add_cycles(pic);
        self.install_bitcoin(pic);
        self.install(pic);
        self.set_controllers(pic);
        canister_id
    }
    /// Deploy to a new pic.
    pub fn deploy(&mut self) -> PicSigner {
        let pic = PocketIcBuilder::new()
            .with_bitcoin_subnet()
            .with_ii_subnet()
            .with_fiduciary_subnet()
            .build();
        let canister_id = self.deploy_to(&pic);
        PicSigner {
            pic: Arc::new(pic),
            canister_id,
        }
    }
}

#[inline]
pub fn controller() -> Principal {
    Principal::from_text(CONTROLLER)
        .expect("Test setup error: Failed to parse controller principal")
}

pub fn setup() -> PicSigner {
    BackendBuilder::default().deploy()
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

/// A test signing canister with a shared reference to the `PocketIc` instance it is installed on.
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

/// Common methods for interacting with a canister using `PocketIc`.
pub trait PicCanisterTrait {
    /// A shared PocketIc instance.
    ///
    /// Note: `PocketIc` uses interior mutability for query and update calls.  No external mut annotation or locks appear to be necessary.
    fn pic(&self) -> Arc<PocketIc>;

    /// The ID of this canister.
    fn canister_id(&self) -> Principal;

    /// Makes an update call to the canister.
    fn update<T>(&self, caller: Principal, method: &str, arg: impl CandidType) -> Result<T, String>
    where
        T: for<'a> Deserialize<'a> + CandidType,
    {
        self.pic()
            .update_call(self.canister_id(), caller, method, encode_one(arg).unwrap())
            .map_err(|e| {
                format!(
                    "Update call error. RejectionCode: {:?}, Error: {}",
                    e.code, e.description
                )
            })
            .and_then(|reply| match reply {
                WasmResult::Reply(reply) => {
                    decode_one(&reply).map_err(|e| format!("Decoding failed: {e}"))
                }
                WasmResult::Reject(error) => Err(error),
            })
    }

    /// Makes a query call to the canister.
    #[allow(dead_code)]
    fn query<T>(&self, caller: Principal, method: &str, arg: impl CandidType) -> Result<T, String>
    where
        T: for<'a> Deserialize<'a> + CandidType,
    {
        self.pic()
            .query_call(self.canister_id(), caller, method, encode_one(arg).unwrap())
            .map_err(|e| {
                format!(
                    "Query call error. RejectionCode: {:?}, Error: {}",
                    e.code, e.description
                )
            })
            .and_then(|reply| match reply {
                WasmResult::Reply(reply) => {
                    decode_one(&reply).map_err(|_| "Decoding failed".to_string())
                }
                WasmResult::Reject(error) => Err(error),
            })
    }
}
