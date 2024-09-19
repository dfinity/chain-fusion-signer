use candid::Principal;

/// The cycles ledger canister ID on mainnet.
const MAINNET_CYCLES_LEDGER_CANISTER_ID: &str = "um5iw-rqaaa-aaaaq-qaaba-cai";

/// The cycles ledger canister ID.
///
/// - If a `cycles_ledger` canister is listed in `dfx.json`, the `dfx build` command will set the
///   environment variable `CANISTER_ID_CYCLES_LEDGER` and we use this to obtain the canister ID.
/// - Otherwise, the mainnet cycles ledger canister ID is used.
const CYCLES_LEDGER_CANISTER_ID: &str = if let Some(id) = option_env!("CANISTER_ID_CYCLES_LEDGER") {
    id
} else {
    MAINNET_CYCLES_LEDGER_CANISTER_ID
};

/// The `CYCLES_LEDGER_CANISTER_ID` as a `Principal`.
///
/// # Panics
/// - If the `CYCLES_LEDGER_CANISTER_ID` is not a valid `Principal`.
#[must_use]
pub fn cycles_ledger_canister_id() -> Principal {
    Principal::from_text(CYCLES_LEDGER_CANISTER_ID)
        .expect("Invalid cycles ledger canister ID provided at compile time.")
}
