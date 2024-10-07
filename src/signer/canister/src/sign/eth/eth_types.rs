use candid::{CandidType, Deserialize, Nat, Principal};
use ic_cdk::api::call::RejectionCode;

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct EthAddressRequest {
    /// The principal owning the eth address.
    pub principal: Principal,
}
#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct EthAddressResponse {
    /// The eth address.
    pub address: String,
}
#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum EthAddressError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An `ic_cdk::call::CallResult` error received when making the canister thereshold signature API call.
    SigningError(RejectionCode, String),
}
impl From<ic_papi_api::PaymentError> for EthAddressError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}
impl From<(RejectionCode, String)> for EthAddressError {
    fn from((rejection_code, message): (RejectionCode, String)) -> Self {
        Self::SigningError(rejection_code, message)
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum EthAddressOfCallerError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An `ic_cdk::call::CallResult` error received when making the canister thereshold signature API call.
    SigningError(RejectionCode, String),
}
impl From<ic_papi_api::PaymentError> for EthAddressOfCallerError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}
impl From<(RejectionCode, String)> for EthAddressOfCallerError {
    fn from((rejection_code, message): (RejectionCode, String)) -> Self {
        Self::SigningError(rejection_code, message)
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub struct EthSignTransactionRequest {
    pub chain_id: Nat,
    pub to: String,
    pub gas: Nat,
    pub max_fee_per_gas: Nat,
    pub max_priority_fee_per_gas: Nat,
    pub value: Nat,
    pub nonce: Nat,
    pub data: Option<String>,
}
// Note: This is the same type, but copied rather than renamed to avoid breaking the API.
// TODO: Delete `SignRequest` once the unpaid APIs have been deleted.
impl From<EthSignTransactionRequest>
    for ic_chain_fusion_signer_api::types::transaction::SignRequest
{
    fn from(req: EthSignTransactionRequest) -> Self {
        Self {
            chain_id: req.chain_id,
            to: req.to,
            gas: req.gas,
            max_fee_per_gas: req.max_fee_per_gas,
            max_priority_fee_per_gas: req.max_priority_fee_per_gas,
            value: req.value,
            nonce: req.nonce,
            data: req.data,
        }
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum EthSignTransactionError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An `ic_cdk::call::CallResult` error received when making the canister thereshold signature API call.
    SigningError(RejectionCode, String),
}
impl From<ic_papi_api::PaymentError> for EthSignTransactionError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}
impl From<(RejectionCode, String)> for EthSignTransactionError {
    fn from((rejection_code, message): (RejectionCode, String)) -> Self {
        Self::SigningError(rejection_code, message)
    }
}
