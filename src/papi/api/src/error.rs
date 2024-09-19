//! Payment API error types.
use candid::{CandidType, Deserialize, Principal};
pub use cycles_ledger_client::Account;
use cycles_ledger_client::WithdrawFromError;

use crate::caller::TokenAmount;

#[derive(Debug, CandidType, Deserialize, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum PaymentError {
    UnsupportedPaymentType,
    LedgerUnreachable {
        ledger: Principal,
    },
    LedgerError {
        ledger: Principal,
        error: WithdrawFromError,
    },
    InsufficientFunds {
        needed: TokenAmount,
        available: TokenAmount,
    },
}
