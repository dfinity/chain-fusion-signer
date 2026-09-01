//! Types for the Schnorr signing API.
use candid::{CandidType, Deserialize};

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum SchnorrSigningError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An inter-canister call error from the threshold signature API.
    SigningError(String),
    /// A caller-supplied argument exceeds the documented size limits.
    ///
    /// The derivation path may have at most 253 elements totalling at most 4096 bytes,
    /// and the key name at most 128 bytes.  See [`crate::limits`] for the exact limits.
    InvalidArgument { msg: String },
}
impl From<ic_papi_api::PaymentError> for SchnorrSigningError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}
impl From<ic_cdk_management_canister::SignCallError> for SchnorrSigningError {
    fn from(e: ic_cdk_management_canister::SignCallError) -> Self {
        Self::SigningError(e.to_string())
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum SchnorrPublicKeyError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An inter-canister call error from the threshold signature API.
    SigningError(String),
    /// A caller-supplied argument exceeds the documented size limits.
    ///
    /// The derivation path may have at most 253 elements totalling at most 4096 bytes,
    /// and the key name at most 128 bytes.  See [`crate::limits`] for the exact limits.
    InvalidArgument { msg: String },
}
impl From<ic_papi_api::PaymentError> for SchnorrPublicKeyError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}
impl From<ic_cdk::call::Error> for SchnorrPublicKeyError {
    fn from(e: ic_cdk::call::Error) -> Self {
        Self::SigningError(e.to_string())
    }
}

#[derive(CandidType, Deserialize, Debug, Clone)]
pub enum SchnorrSignWithEcdsaError {
    /// Payment failed.
    PaymentError(ic_papi_api::PaymentError),
    /// An inter-canister call error from the threshold signature API.
    SigningError(String),
}
impl From<ic_papi_api::PaymentError> for SchnorrSignWithEcdsaError {
    fn from(e: ic_papi_api::PaymentError) -> Self {
        Self::PaymentError(e)
    }
}
