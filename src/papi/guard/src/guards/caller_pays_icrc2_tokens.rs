//! Code to receive any ICRC-2 token as payment.

// Well known ICRC-2 tokens
// TODO

use super::{PaymentError, PaymentGuard};
use candid::{Nat, Principal};
use cycles_ledger_client::TransferFromArgs;
use ic_papi_api::{caller::TokenAmount, Account};

pub struct CallerPaysIcrc2TokensPaymentGuard {
    /// The ledger for that specific token
    pub ledger: Principal,
}

impl PaymentGuard for CallerPaysIcrc2TokensPaymentGuard {
    async fn deduct(&self, cost: TokenAmount) -> Result<(), PaymentError> {
        let caller = ic_cdk::api::caller();
        cycles_ledger_client::Service(self.ledger)
            .icrc_2_transfer_from(&TransferFromArgs {
                from: Account {
                    owner: caller,
                    subaccount: None,
                },
                to: Account {
                    owner: ic_cdk::api::id(),
                    subaccount: None,
                },
                amount: Nat::from(cost),
                spender_subaccount: None,
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
