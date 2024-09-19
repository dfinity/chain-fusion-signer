//! Guards for specific flows

use candid::Principal;
use ic_papi_api::{caller::TokenAmount, PaymentError, PaymentType};
pub mod any;
pub mod attached_cycles;
pub mod icrc2_cycles;

#[allow(async_fn_in_trait)]
pub trait PaymentGuard {
    async fn deduct(&self, fee: TokenAmount) -> Result<(), PaymentError>;
}

#[allow(async_fn_in_trait)]
pub trait PaymentGuard2 {
    async fn deduct(
        &self,
        context: PaymentContext,
        payment: PaymentType,
        fee: TokenAmount,
    ) -> Result<(), PaymentError>;
}

pub struct PaymentContext {
    own_canister_id: Principal,
    caller: Principal,
}
impl Default for PaymentContext {
    fn default() -> Self {
        Self {
            own_canister_id: ic_cdk::api::id(),
            caller: ic_cdk::caller(),
        }
    }
}
