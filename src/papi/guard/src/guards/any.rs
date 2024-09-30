//! Accepts any payment that the vendor accepts.

use candid::{CandidType, Deserialize, Principal};
use ic_papi_api::{
    caller::{CallerPaysIcrc2Tokens, PatronPaysIcrc2Cycles, PatronPaysIcrc2Tokens, TokenAmount},
    principal2account, Account, PaymentError, PaymentType,
};

use super::{
    attached_cycles::AttachedCyclesPayment,
    caller_pays_icrc2_tokens::CallerPaysIcrc2TokensPaymentGuard,
    icrc2_cycles::Icrc2CyclesPaymentGuard,
    patron_pays_icrc2_tokens::PatronPaysIcrc2TokensPaymentGuard, PaymentContext, PaymentGuard,
    PaymentGuard2,
};

/// A guard that accepts a user-specified payment type, providing the vendor supports it.
pub struct AnyPaymentGuard<const CAP: usize> {
    pub supported: [VendorPaymentConfig; CAP],
}

/// Vendor payment configuration, including details that may not necessarily be shared with the customer.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum VendorPaymentConfig {
    /// Cycles are received by the vendor canister.
    AttachedCycles,
    /// Cycles are received by the vendor canister.
    CallerPaysIcrc2Cycles,
    /// Cycles are received by the vendor canister.
    PatronPaysIcrc2Cycles,
    /// The caller pays tokens to the vendor's main account on the chosen ledger.
    CallerPaysIcrc2Tokens { ledger: Principal },
    /// A patron pays tokens to a subaccount belonging to the vendor on the chosen ledger.
    /// - The vendor needs to move the tokens to their main account.
    PatronPaysIcrc2Tokens { ledger: Principal },
}

/// A user's requested payment type paired with a vendor's configuration.
#[derive(Debug, Clone, Eq, PartialEq, CandidType, Deserialize)]
pub enum PaymentWithConfig {
    AttachedCycles,
    CallerPaysIcrc2Cycles,
    PatronPaysIcrc2Cycles(PatronPaysIcrc2Cycles),
    CallerPaysIcrc2Tokens(CallerPaysIcrc2Tokens),
    PatronPaysIcrc2Tokens(PatronPaysIcrc2Tokens),
}

impl<const CAP: usize> PaymentGuard2 for AnyPaymentGuard<CAP> {
    async fn deduct(
        &self,
        context: PaymentContext,
        payment: PaymentType,
        fee: TokenAmount,
    ) -> Result<(), PaymentError> {
        let PaymentContext {
            caller,
            own_canister_id,
        } = context;
        let payment_config = self
            .config(payment)
            .ok_or(PaymentError::UnsupportedPaymentType)?;
        match payment_config {
            PaymentWithConfig::AttachedCycles => AttachedCyclesPayment {}.deduct(fee).await,
            PaymentWithConfig::CallerPaysIcrc2Cycles => {
                Icrc2CyclesPaymentGuard {
                    payer_account: Account {
                        owner: caller,
                        subaccount: None,
                    },
                    spender_subaccount: None,
                    own_canister_id,
                }
                .deduct(fee)
                .await
            }
            PaymentWithConfig::PatronPaysIcrc2Cycles(patron) => {
                Icrc2CyclesPaymentGuard {
                    payer_account: patron,
                    spender_subaccount: Some(principal2account(&caller)),
                    ..Icrc2CyclesPaymentGuard::default()
                }
                .deduct(fee)
                .await
            }
            PaymentWithConfig::CallerPaysIcrc2Tokens(CallerPaysIcrc2Tokens { ledger }) => {
                CallerPaysIcrc2TokensPaymentGuard { ledger }
                    .deduct(fee)
                    .await
            }
            PaymentWithConfig::PatronPaysIcrc2Tokens(payment_type) => {
                PatronPaysIcrc2TokensPaymentGuard {
                    ledger: payment_type.ledger,
                    payer_account: payment_type.patron,
                    spender_subaccount: Some(principal2account(&caller)),
                    own_canister_id,
                }
                .deduct(fee)
                .await
            }
        }
    }
}
impl<const CAP: usize> AnyPaymentGuard<CAP> {
    /// Find the vendor configuration for the offered payment type.
    #[must_use]
    pub fn config(&self, payment: PaymentType) -> Option<PaymentWithConfig> {
        match payment {
            PaymentType::AttachedCycles => self
                .supported
                .iter()
                .find(|&x| *x == VendorPaymentConfig::AttachedCycles)
                .map(|_| PaymentWithConfig::AttachedCycles),
            PaymentType::CallerPaysIcrc2Cycles => self
                .supported
                .iter()
                .find(|&x| *x == VendorPaymentConfig::CallerPaysIcrc2Cycles)
                .map(|_| PaymentWithConfig::CallerPaysIcrc2Cycles),
            PaymentType::PatronPaysIcrc2Cycles(patron) => self
                .supported
                .iter()
                .find(|&x| *x == VendorPaymentConfig::PatronPaysIcrc2Cycles)
                .map(|_| PaymentWithConfig::PatronPaysIcrc2Cycles(patron)),
            PaymentType::CallerPaysIcrc2Tokens(payment_type) => self
                .supported
                .iter()
                .find(|&x| {
                    *x == VendorPaymentConfig::CallerPaysIcrc2Tokens {
                        ledger: payment_type.ledger,
                    }
                })
                .map(|_| PaymentWithConfig::CallerPaysIcrc2Tokens(payment_type)),
            PaymentType::PatronPaysIcrc2Tokens(payment_type) => self
                .supported
                .iter()
                .find(|&x| {
                    *x == VendorPaymentConfig::PatronPaysIcrc2Tokens {
                        ledger: payment_type.ledger,
                    }
                })
                .map(|_| PaymentWithConfig::PatronPaysIcrc2Tokens(payment_type)),
            _ => None,
        }
    }
}
