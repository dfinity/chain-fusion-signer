//! A generic signing API equivalent to that provided by the canister API.
use crate::derivation_path::Schema;
use candid::{CandidType, Deserialize};
use ic_cdk::api::call::RejectionCode;
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaPublicKeyArgument, EcdsaPublicKeyResponse,
    SignWithEcdsaArgument, SignWithEcdsaResponse,
};

/// Signs a message with a generic ECDSA key for the user.
///
/// Warning: The user supplied derivation path is used as-is.  The caller is responsible for ensuring that unintended sub-keys are not requested.
pub async fn generic_ecdsa_public_key(
    mut arg: EcdsaPublicKeyArgument,
) -> Result<(EcdsaPublicKeyResponse,), GenericSigningError> {
    arg.derivation_path =
        Schema::Generic.derivation_path_ending_in(&ic_cdk::caller(), arg.derivation_path);
    ecdsa_public_key(arg)
        .await
        .map_err(|(rejection_code, message)| {
            GenericSigningError::SigningError(rejection_code, message)
        })
}

pub async fn generic_sign_with_ecdsa(
    mut arg: SignWithEcdsaArgument,
) -> Result<(SignWithEcdsaResponse,), GenericSigningError> {
    arg.derivation_path =
        Schema::Generic.derivation_path_ending_in(&ic_cdk::caller(), arg.derivation_path);
    sign_with_ecdsa(arg)
        .await
        .map_err(|(rejection_code, message)| {
            GenericSigningError::SigningError(rejection_code, message)
        })
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
