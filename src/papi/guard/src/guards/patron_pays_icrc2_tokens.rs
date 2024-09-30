//! Code to receive cycles as payment, credited to the canister, using ICRC-2 and a cycles-ledger specific withdrawal method.
use super::{PaymentError, PaymentGuard};
use candid::{Nat, Principal};
use cycles_ledger_client::TransferFromArgs;
use ic_papi_api::{caller::TokenAmount, Account};

/// Accepts cycles using an ICRC-2 approve followed by withdrawing the cycles to the current canister.  Withdrawing
/// cycles to the current canister is specific to the cycles ledger canister; it is not part of the ICRC-2 standard.
pub struct PatronPaysIcrc2TokensPaymentGuard {
    /// The ledger for that specific token
    pub ledger: Principal,
    /// The payer
    pub payer_account: Account,
    /// The spender, if different from the payer.
    pub spender_subaccount: Option<serde_bytes::ByteBuf>,
    /// Own canister ID
    pub own_canister_id: Principal,
}
impl PatronPaysIcrc2TokensPaymentGuard {
    #[must_use]
    pub fn default_account() -> Account {
        Account {
            owner: ic_cdk::caller(),
            subaccount: None,
        }
    }
}

impl PaymentGuard for PatronPaysIcrc2TokensPaymentGuard {
    async fn deduct(&self, cost: TokenAmount) -> Result<(), PaymentError> {
        // Note: The cycles ledger client is ICRC-2 compatible so can be used here.
        cycles_ledger_client::Service(self.ledger)
            .icrc_2_transfer_from(&TransferFromArgs {
                from: self.payer_account.clone(),
                to: Account {
                    owner: ic_cdk::api::id(),
                    subaccount: None,
                },
                amount: Nat::from(cost),
                spender_subaccount: self.spender_subaccount.clone(),
                created_at_time: None,
                memo: None,
                fee: None,
            })
            .await
            .map_err(|(rejection_code, string)| {
                eprintln!(
                    "Failed to reach ledger canister at {}: {rejection_code:?}: {string}",
                    self.ledger
                );
                PaymentError::LedgerUnreachable {
                    ledger: self.ledger,
                }
            })?
            .0
            .map_err(|error| {
                eprintln!(
                    "Failed to withdraw from ledger canister at {}: {error:?}",
                    self.ledger
                );
                PaymentError::LedgerTransferFromError {
                    ledger: self.ledger,
                    error,
                }
            })
            .map(|_| ())
    }
}
