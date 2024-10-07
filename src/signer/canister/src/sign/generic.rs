//! A generic signing API equivalent to that provided by the canister API.
use crate::derivation_path::Schema;
use candid::{CandidType, Deserialize};
use ic_cdk::api::call::RejectionCode;
use ic_cdk::api::management_canister::ecdsa as canister_ecdsa;
use ic_cdk::api::management_canister::ecdsa::{
    EcdsaPublicKeyArgument, EcdsaPublicKeyResponse, SignWithEcdsaArgument, SignWithEcdsaResponse,
};

/// Signs a message with a generic ECDSA key for the user.
///
/// Warning: The user supplied derivation path is used as-is.  The caller is responsible for ensuring that unintended sub-keys are not requested.
pub async fn caller_ecdsa_public_key(
    mut arg: EcdsaPublicKeyArgument,
) -> Result<(EcdsaPublicKeyResponse,), GenericCallerEcdsaPublicKeyError> {
    arg.derivation_path =
        Schema::Generic.derivation_path_ending_in(&ic_cdk::caller(), arg.derivation_path);
    Ok(canister_ecdsa::ecdsa_public_key(arg).await?)
}

pub async fn sign_with_ecdsa(
    mut arg: SignWithEcdsaArgument,
) -> Result<(SignWithEcdsaResponse,), GenericSignWithEcdsaError> {
    arg.derivation_path =
        Schema::Generic.derivation_path_ending_in(&ic_cdk::caller(), arg.derivation_path);
    Ok(canister_ecdsa::sign_with_ecdsa(arg).await?)
}

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
