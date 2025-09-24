//! Support for the standard `get_canister_status` method returning a `CanisterStatusResultV2`.
//!
//! Note: This API is used my many canisters but the code is not packaged up in a portable way and
//! implementations typically use old APIs to get the data.
//!
//! The `ic_cdk` has a method called [`canister_status`](https://docs.rs/ic-cdk/0.18.0/ic_cdk/management_canister/fn.canister_status.html)
//! with all the same data. Consumers such as the cycle management canister should consider
//! supporting that. In the meantime we convert the current `ic_cdk` response into the
//! currently requested `CanisterStatusResultV2`.

use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::management_canister::{
    canister_status, CanisterStatusArgs, CanisterStatusType, DefiniteCanisterSettings,
};

/// Copy of the synonymous Rosetta type.
#[derive(CandidType, Debug, Deserialize, Eq, PartialEq)]
pub struct CanisterStatusResultV2 {
    status: CanisterStatusType,
    module_hash: Option<Vec<u8>>,
    controller: Principal,
    settings: DefiniteCanisterSettingsArgs,
    memory_size: Nat,
    cycles: Nat,
    // this is for compat with Spec 0.12/0.13
    balance: Vec<(Vec<u8>, Nat)>,
    freezing_threshold: Nat,
    idle_cycles_burned_per_day: Nat,
}

/// Copy of synonymous Rosetta type.
#[derive(CandidType, Deserialize, Debug, Eq, PartialEq)]
pub struct DefiniteCanisterSettingsArgs {
    controller: Principal,
    controllers: Vec<Principal>,
    compute_allocation: Nat,
    memory_allocation: Nat,
    freezing_threshold: Nat,
}

impl TryFrom<DefiniteCanisterSettings> for DefiniteCanisterSettingsArgs {
    type Error = &'static str;
    fn try_from(value: DefiniteCanisterSettings) -> Result<Self, Self::Error> {
        let DefiniteCanisterSettings {
            controllers,
            compute_allocation,
            memory_allocation,
            freezing_threshold,
            // TODO: should API method get_canister_status be extended with additional information
            // such as reserved_cycles_limit, log_visibility, or wasm_memory_limit?
            ..
        } = value;
        Ok(Self {
            controller: *controllers
                .first()
                .ok_or("This canister has not even one controller")?,
            controllers,
            compute_allocation,
            memory_allocation,
            freezing_threshold,
        })
    }
}

/// Gets status information about the canister.
///
/// See [IC method `canister_status`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-canister_status).
///
/// # Panics
/// - If the call to the management canister fails.
/// - If the response cannot be converted to `CanisterStatusResultV2`.  For example, it looks as if
///   it will panic if the canister has no controllers.
pub async fn get_canister_status_v2() -> CanisterStatusResultV2 {
    let canister_id = ic_cdk::api::canister_self(); // Own canister ID.
    
    // canister_status returns a Result containing the response
    let result = canister_status(&CanisterStatusArgs { canister_id })
        .await
        .unwrap_or_else(|err| panic!("Failed to get status: {err:#?}"));

    let controller = *result.settings
        .controllers
        .first()
        .expect("This canister has not even one controller");
    
    let balance = vec![(vec![0], result.cycles.clone())];
    let freezing_threshold = result.settings.freezing_threshold.clone();

    let settings = DefiniteCanisterSettingsArgs::try_from(result.settings)
        .expect("Failed to convert DefiniteCanisterSettings");

    CanisterStatusResultV2 {
        status: result.status,
        module_hash: result.module_hash,
        controller,
        settings,
        memory_size: result.memory_size,
        cycles: result.cycles,
        balance,
        freezing_threshold,
        idle_cycles_burned_per_day: result.idle_cycles_burned_per_day,
    }
}
