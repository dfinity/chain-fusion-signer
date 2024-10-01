//! Types used primarily by the caller of the payment API.
use candid::{CandidType, Deserialize, Principal};
pub use cycles_ledger_client::Account;

/// How a caller states that they will pay.
#[derive(Debug, CandidType, Deserialize, Clone, Eq, PartialEq)]
#[non_exhaustive]
pub enum PaymentType {
    /// The caller is paying with cycles attached to the call.
    ///
    /// Note: This is available to inter-canister aclls only; not to ingress messages.
    ///
    /// Note: The API does not require additional arguments to support this payment type.
    AttachedCycles,
    /// The caller is paying with cycles from their main account on the cycles ledger.
    CallerPaysIcrc2Cycles,
    /// A patron is paying with cycles on behalf of the caller.
    PatronPaysIcrc2Cycles(PatronPaysIcrc2Cycles),
    /// The caller is paying with tokens from their main account on the specified ledger.
    CallerPaysIcrc2Tokens(CallerPaysIcrc2Tokens),
    /// A patron is paying, on behalf of the caller, from an account on the specified ledger.
    PatronPaysIcrc2Tokens(PatronPaysIcrc2Tokens),
}

pub type PatronPaysIcrc2Cycles = Account;

#[derive(Debug, CandidType, Deserialize, Copy, Clone, Eq, PartialEq)]
pub struct CallerPaysIcrc2Tokens {
    pub ledger: Principal,
}

#[derive(Debug, CandidType, Deserialize, Clone, Eq, PartialEq)]
pub struct PatronPaysIcrc2Tokens {
    pub ledger: Principal,
    pub patron: Account,
}

pub type TokenAmount = u64;
