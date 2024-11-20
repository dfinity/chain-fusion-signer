use candid::{CandidType, Deserialize};
use ic_cdk::api::call::RejectionCode;

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum GenericSigningError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An `ic_cdk::call::CallResult` error received when making the canister thereshold signature API call.
    SigningError(RejectionCode, String),
}

impl From<ic_papi_api::PaymentError> for GenericSigningError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum SignerSchnorrPublicKeyError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    // Signing Failed
    SigningError,
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum GenericCallerEcdsaPublicKeyError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An `ic_cdk::call::CallResult` error received when making the canister thereshold signature API call.
    SigningError(RejectionCode, String),
}
impl From<ic_papi_api::PaymentError> for GenericCallerEcdsaPublicKeyError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}
impl From<(RejectionCode, String)> for GenericCallerEcdsaPublicKeyError {
    fn from((rejection_code, message): (RejectionCode, String)) -> Self {
        Self::SigningError(rejection_code, message)
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum GenericSignWithEcdsaError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An `ic_cdk::call::CallResult` error received when making the canister thereshold signature API call.
    SigningError(RejectionCode, String),
}
impl From<ic_papi_api::PaymentError> for GenericSignWithEcdsaError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}
impl From<(RejectionCode, String)> for GenericSignWithEcdsaError {
    fn from((rejection_code, message): (RejectionCode, String)) -> Self {
        Self::SigningError(rejection_code, message)
    }
}
