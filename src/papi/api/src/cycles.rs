use candid::Principal;

/// The cycles ledger canister ID on mainnet.
///
/// Note: This canister ID should also be used in test environments.  The dfx.json
/// can implement this with:
/// ```
/// { "canisters": {
///   "cycles_ledger": {
///     ...
///     "specified_id": "um5iw-rqaaa-aaaaq-qaaba-cai",
///     "remote": {
///        "id": {
///                "ic": "um5iw-rqaaa-aaaaq-qaaba-cai"
///        }
///     }
///   },
///  ...
/// }
/// ```
pub const MAINNET_CYCLES_LEDGER_CANISTER_ID: &str = "um5iw-rqaaa-aaaaq-qaaba-cai";

/// The `MAINNET_CYCLES_LEDGER_CANISTER_ID` as a `Principal`.
///
/// # Panics
/// - If the `MAINNET_CYCLES_LEDGER_CANISTER_ID` is not a valid `Principal`.
#[must_use]
pub fn cycles_ledger_canister_id() -> Principal {
    Principal::from_text(MAINNET_CYCLES_LEDGER_CANISTER_ID)
        .expect("Invalid cycles ledger canister ID provided at compile time.")
}
