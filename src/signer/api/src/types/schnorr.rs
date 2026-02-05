//! Types for the Schnorr signing API.
use candid::{CandidType, Deserialize};
use super::RejectCode;

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum SchnorrSigningError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An `ic_cdk::call::CallResult` error received when making the canister threshold signature
    /// API call.
    SigningError(RejectCode, String),
}
impl From<ic_papi_api::PaymentError> for SchnorrSigningError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}
impl From<(RejectCode, String)> for SchnorrSigningError {
    fn from((rejection_code, message): (RejectCode, String)) -> Self {
        Self::SigningError(rejection_code, message)
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum SchnorrPublicKeyError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An `ic_cdk::call::CallResult` error received when making the canister threshold signature
    /// API call.
    SigningError(RejectCode, String),
}
impl From<ic_papi_api::PaymentError> for SchnorrPublicKeyError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}
impl From<(RejectCode, String)> for SchnorrPublicKeyError {
    fn from((rejection_code, message): (RejectCode, String)) -> Self {
        Self::SigningError(rejection_code, message)
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum SchnorrSignWithEcdsaError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An `ic_cdk::call::CallResult` error received when making the canister threshold signature
    /// API call.
    SigningError(RejectCode, String),
}
impl From<ic_papi_api::PaymentError> for SchnorrSignWithEcdsaError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}
