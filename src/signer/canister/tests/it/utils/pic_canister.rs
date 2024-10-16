//! Common methods for interacting with a canister using `PocketIc`.
use candid::{decode_one, encode_one, CandidType, Deserialize, Principal};
use pocket_ic::{PocketIc, WasmResult};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

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
/// The path to a typical Cargo Wasm build.
#[allow(dead_code)]
fn cargo_wasm_path(name: &str) -> String {
    let workspace_dir = workspace_dir();
    workspace_dir
        .join("target/wasm32-unknown-unknown/release")
        .join(name)
        .with_extension("wasm")
        .to_str()
        .unwrap()
        .to_string()
}
/// The path to the wasm after `dfx deploy`.  Expects the Wasm to be gzipped.
///
/// If not already gzipped, please add this to the canister declaration in `dfx.json`: `"gzip": true`
#[allow(dead_code)]
fn dfx_wasm_path(name: &str) -> String {
    workspace_dir()
        .join(format!(".dfx/local/canisters/{name}/{name}.wasm.gz"))
        .to_str()
        .unwrap()
        .to_string()
}

/// A typical canister running on PocketIC.
pub struct PicCanister {
    pub pic: Arc<PocketIc>,
    pub canister_id: Principal,
}

impl PicCanisterTrait for PicCanister {
    /// The shared PocketIc instance.
    fn pic(&self) -> Arc<PocketIc> {
        self.pic.clone()
    }
    /// The ID of this canister.
    fn canister_id(&self) -> Principal {
        self.canister_id.clone()
    }
}

impl PicCanister {
    /// Creates a new canister.
    pub fn new(pic: Arc<PocketIc>, wasm_path: &str) -> Self {
        PicCanisterBuilder::default()
            .with_wasm(wasm_path)
            .deploy_to(pic)
    }
}

/// Canister installer, using the builder pattern, for use in test environmens using `PocketIC`.
///
/// # Example
/// For a simple test environment consisting of a canister deployed to a pocket-ic:
/// ```
/// // Before testing, deploy the canisters to local.  Ensure that the Wasms are compressed.
/// // This ensures that files are present and well formed.
/// // A simple test environment consisting of a pocket_ic with the wasm.gz deployed to it can then be created with:
/// let pic_canister = PicCanisterBuilder::new("my_canister_name").deploy();
/// ```
/// To add a canister to an existing `PocketIC`:
/// ```
/// let pic = PocketIc::new();
/// let canister_id = PicCanisterBuilder::default().deploy_to(&pic);
/// ```
/// To redeploy an existing canister:
/// ```
/// // First deployment:
/// let pic_canister = PicCanisterBuilder::default().deploy();
/// // Subsequent deployment:
/// let canister_id = PicCanisterBuilder::default().with_canister(canister_id).deploy_to(&pic);
/// ```
/// To customise the deployment, use the `.with_*` modifiers.  E.g.:
/// ```
/// let (pic, canister_id) = PicCanisterBuilder::default()
///    .with_wasm("path/to/ic_chainfusion_signer.wasm")
///    .with_arg(vec![1, 2, 3])
///    .with_controllers(vec![Principal::from_text("controller").unwrap()])
///    .with_cycles(1_000_000_000_000)
///    .deploy();
/// ```
#[derive(Debug)]
pub struct PicCanisterBuilder {
    /// Canister name, as it appears in dfx.json.
    #[allow(dead_code)] // Useful in debug printouts.
    canister_name: Option<String>,
    /// Canister ID of the canister.  If not set, a new canister will be created.
    canister_id: Option<Principal>,
    /// Cycles to add to the canister.
    cycles: u128,
    /// Path to the wasm file.
    wasm_path: String,
    /// Argument to pass to the canister.
    arg: Vec<u8>,
    /// Controllers of the canister.
    ///
    /// If the list is not specified, controllers will be unchnaged from the PocketIc defaults.
    controllers: Option<Vec<Principal>>,
}
// Defaults
impl PicCanisterBuilder {
    /// The default number of cycles to add to the canister on deployment.
    ///
    /// To override, please use `with_cycles()`.
    pub const DEFAULT_CYCLES: u128 = 2_000_000_000_000;
    /// The default argument to pass to the canister:  `()`.
    ///
    /// To override, please use `with_arg()`.
    pub fn default_arg() -> Vec<u8> {
        encode_one(()).unwrap()
    }
}
impl Default for PicCanisterBuilder {
    fn default() -> Self {
        Self {
            canister_name: None,
            canister_id: None,
            cycles: Self::DEFAULT_CYCLES,
            wasm_path: "unspecified.wasm".to_string(),
            arg: Self::default_arg(),
            controllers: None,
        }
    }
}
// Customisation
impl PicCanisterBuilder {
    /// Create a new canister builder.
    #[allow(dead_code)]
    fn new(name: &str) -> Self {
        Self {
            canister_name: Some(name.to_string()),
            canister_id: None,
            cycles: Self::DEFAULT_CYCLES,
            wasm_path: dfx_wasm_path(name),
            arg: Self::default_arg(),
            controllers: None,
        }
    }
    /// Sets a custom argument for the canister.
    #[allow(dead_code)]
    pub fn with_arg(mut self, arg: Vec<u8>) -> Self {
        self.arg = arg;
        self
    }
    /// Deploys to an existing canister with the given ID.
    #[allow(dead_code)]
    pub fn with_canister(mut self, canister_id: Principal) -> Self {
        self.canister_id = Some(canister_id);
        self
    }
    /// Sets custom controllers for the canister.
    #[allow(dead_code)]
    pub fn with_controllers(mut self, controllers: Vec<Principal>) -> Self {
        self.controllers = Some(controllers);
        self
    }
    /// Sets the cycles to add to the canister.
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
impl PicCanisterBuilder {
    /// Reads the Wasm bytes from the configured path.
    fn wasm_bytes(&self) -> Vec<u8> {
        fs::read(self.wasm_path.clone()).expect(&format!("Could not find wasm: {}", self.wasm_path))
    }
}
// Builder
impl PicCanisterBuilder {
    /// Get or create canister.
    fn canister_id(&mut self, pic: &PocketIc) -> Principal {
        if let Some(canister_id) = self.canister_id {
            canister_id
        } else {
            let canister_id = pic.create_canister();
            self.canister_id = Some(canister_id);
            canister_id
        }
    }
    /// Add cycles to the canister.
    fn add_cycles(&mut self, pic: &PocketIc) {
        if self.cycles > 0 {
            let canister_id = self.canister_id(pic);
            pic.add_cycles(canister_id, self.cycles);
        }
    }
    /// Install the canister.
    fn install(&mut self, pic: &PocketIc) {
        let wasm_bytes = self.wasm_bytes();
        let canister_id = self.canister_id(pic);
        let arg = self.arg.clone();
        pic.install_canister(canister_id, wasm_bytes, arg, None);
    }
    /// Set controllers of the canister.
    fn set_controllers(&mut self, pic: &PocketIc) {
        if let Some(controllers) = self.controllers.clone() {
            let canister_id = self.canister_id(pic);
            pic.set_controllers(canister_id.clone(), None, controllers)
                .expect("Test setup error: Failed to set controllers");
        }
    }
    /// Setup the canister.
    pub fn deploy_to(&mut self, pic: Arc<PocketIc>) -> PicCanister {
        let canister_id = self.canister_id(&pic);
        self.add_cycles(&pic);
        self.install(&pic);
        self.set_controllers(&pic);
        PicCanister {
            pic: pic.clone(),
            canister_id,
        }
    }
    /// Deploys the canister to a new PocketIC instance.
    pub fn deploy(&mut self) -> PicCanister {
        let pic = PocketIc::new();
        self.deploy_to(Arc::new(pic))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_deploy_canister() {
        let _pic = PicCanisterBuilder::new("example_backend").deploy();
    }
    struct MulticanisterTestEnv {
        #[allow(dead_code)] // Created in tests.
        pub example_backend: PicCanister,
        #[allow(dead_code)] // Created in tests.
        pub example_frontend: PicCanister,
    }
    impl Default for MulticanisterTestEnv {
        fn default() -> Self {
            let example_backend = PicCanisterBuilder::new("example_backend").deploy();
            // Deploy the frontend to the same pic:
            let pic = example_backend.pic();
            let example_frontend = PicCanisterBuilder::new("example_frontend").deploy_to(pic);
            Self {
                example_backend,
                example_frontend,
            }
        }
    }
    #[test]
    fn can_deploy_multiple_canisters() {
        let _env = MulticanisterTestEnv::default();
    }
}
