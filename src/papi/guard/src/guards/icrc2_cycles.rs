//! Code to receive cycles as payment, credited to the canister, using ICRC-2 and a cycles-ledger specific withdrawal method.
use super::{PaymentError, PaymentGuard};
use candid::{Nat, Principal};
use cycles_ledger_client::WithdrawFromArgs;
use ic_papi_api::{caller::TokenAmount, cycles::cycles_ledger_canister_id, Account};

/// Accepts cycles using an ICRC-2 approve followed by withdrawing the cycles to the current canister.  Withdrawing
/// cycles to the current canister is specific to the cycles ledger canister; it is not part of the ICRC-2 standard.
pub struct Icrc2CyclesPaymentGuard {
    /// The payer
    pub payer_account: Account,
    /// The spender, if different from the payer.
    pub spender_subaccount: Option<serde_bytes::ByteBuf>,
    /// Own canister ID
    pub own_canister_id: Principal,
}
impl Icrc2CyclesPaymentGuard {
    #[must_use]
    pub fn default_account() -> Account {
        Account {
            owner: ic_cdk::caller(),
            subaccount: None,
        }
    }
    /// The normal cycles ledger canister ID.
    ///
    /// - If the cycles ledger is listed in `dfx.json`, a normal `dfx build` will set the
    ///   environment variable `CANISTER_ID_CYCLES_LEDGER` and we use this to obtain the canister ID.
    /// - Otherwise, we use the mainnet cycled ledger canister ID, which is `um5iw-rqaaa-aaaaq-qaaba-cai`.
    ///
    /// # Panics
    /// - If the `CANISTER_ID_CYCLES_LEDGER` environment variable is not a valid canister ID at
    ///   build time.
    #[must_use]
    pub fn default_cycles_ledger() -> Principal {
        Principal::from_text(
            option_env!("CANISTER_ID_CYCLES_LEDGER").unwrap_or("um5iw-rqaaa-aaaaq-qaaba-cai"),
        )
        .expect("Compile error: Failed to parse build env var 'CANISTER_ID_CYCLES_LEDGER' as a canister ID.")
    }
}

impl Default for Icrc2CyclesPaymentGuard {
    fn default() -> Self {
        Self {
            payer_account: Self::default_account(),
            own_canister_id: ic_cdk::api::id(),
            spender_subaccount: None,
        }
    }
}

impl PaymentGuard for Icrc2CyclesPaymentGuard {
    async fn deduct(&self, fee: TokenAmount) -> Result<(), PaymentError> {
        cycles_ledger_client::Service(cycles_ledger_canister_id())
            .withdraw_from(&WithdrawFromArgs {
                to: self.own_canister_id,
                amount: Nat::from(fee),
                from: self.payer_account.clone(),
                spender_subaccount: self.spender_subaccount.clone(),
                created_at_time: None,
            })
            .await
            .map_err(|(rejection_code, string)| {
                eprintln!(
                    "Failed to reach ledger canister at {}: {rejection_code:?}: {string}",
                    cycles_ledger_canister_id()
                );
                PaymentError::LedgerUnreachable {
                    ledger: cycles_ledger_canister_id(),
                }
            })?
            .0
            .map_err(|error| {
                eprintln!(
                    "Failed to withdraw from ledger canister at {}: {error:?}",
                    cycles_ledger_canister_id()
                );
                PaymentError::LedgerWithdrawFromError {
                    ledger: cycles_ledger_canister_id(),
                    error,
                }
            })
            .map(|_| ())
    }
}
