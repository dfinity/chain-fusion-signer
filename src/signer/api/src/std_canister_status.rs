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
    canister_status, CanisterStatusArgs, CanisterStatusResult, CanisterStatusType,
    DefiniteCanisterSettings
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

impl TryFrom<CanisterStatusResult> for CanisterStatusResultV2 {
    type Error = &'static str;
    fn try_from(value: CanisterStatusResult) -> Result<Self, Self::Error> {
        let CanisterStatusResult {
            status,
            module_hash,
            settings,
            memory_size,
            cycles,
            idle_cycles_burned_per_day,
            ..
        } = value;

        let controller = *settings
            .controllers
            .first()
            .ok_or("This canister has not even one controller")?;
        let balance = vec![(vec![0], cycles.clone())];
        let freezing_threshold = settings.freezing_threshold.clone();

        Ok(Self {
            status,
            module_hash,
            controller,
            settings: settings.try_into()?,
            memory_size,
            cycles,
            balance,
            freezing_threshold,
            idle_cycles_burned_per_day,
        })
    }
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

/// Gets the canister status using the standard `CanisterStatusResultV2` format.
///
/// # Panics
/// Panics if the canister status cannot be retrieved or if the response cannot be
/// converted to `CanisterStatusResultV2` format.
pub async fn get_canister_status_v2() -> CanisterStatusResultV2 {
    let canister_id = ic_cdk::api::canister_self(); // Own canister ID.
    canister_status(&CanisterStatusArgs { canister_id })
        .await
        .map_err(|err| format!("Failed to get status: {err:#?}"))
        .and_then(|canister_status_response| {
            CanisterStatusResultV2::try_from(canister_status_response)
                .map_err(|str| format!("CanisterStatusResultV2::try_from failed: {str}"))
        })
        .unwrap_or_else(|err| panic!("Couldn't get canister_status of {canister_id}. Err: {err}"))
}
