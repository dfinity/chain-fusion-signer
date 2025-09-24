//! A generic signing API equivalent to that provided by the canister API.
use candid::Principal;
use ic_cdk::management_canister::{
    ecdsa_public_key as ic_ecdsa_public_key, sign_with_ecdsa as ic_sign_with_ecdsa, 
    schnorr_public_key as ic_schnorr_public_key, sign_with_schnorr as ic_sign_with_schnorr,
    EcdsaPublicKeyArgs, EcdsaPublicKeyResult, SignWithEcdsaArgs, SignWithEcdsaResult,
    SchnorrPublicKeyArgs, SchnorrPublicKeyResult, SignWithSchnorrArgs, SignWithSchnorrResult,
};
pub use ic_chain_fusion_signer_api::types::generic::{
    GenericCallerEcdsaPublicKeyError, GenericSignWithEcdsaError,
};
use ic_chain_fusion_signer_api::types::{schnorr::{SchnorrPublicKeyError, SchnorrSigningError}, RejectionCode};

use crate::derivation_path::Schema;

/// A generic ECDSA public key for the user.
///
/// Warning: The user supplied derivation path is used as-is. The caller is responsible for
/// ensuring that unintended subkeys are not requested.
pub async fn caller_ecdsa_public_key(
    mut arg: EcdsaPublicKeyArgs,
) -> Result<EcdsaPublicKeyResult, GenericCallerEcdsaPublicKeyError> {
    arg.derivation_path =
        Schema::Generic.derivation_path_ending_in(&ic_cdk::api::msg_caller(), arg.derivation_path);
    match ic_ecdsa_public_key(&arg).await {
        Ok(result) => Ok(result),
        Err(call_error) => {
            // Use a default rejection code and convert error to string
            Err(GenericCallerEcdsaPublicKeyError::SigningError(
                RejectionCode::CanisterError,
                format!("{call_error:?}")
            ))
        }
    }
}

/// Signs a message with a generic ECDSA key for the user.
///
/// Warning: The user supplied derivation path is used as-is. The caller is responsible for
/// ensuring that unintended subkeys are not requested.
pub async fn sign_with_ecdsa(
    mut arg: SignWithEcdsaArgs,
) -> Result<SignWithEcdsaResult, GenericSignWithEcdsaError> {
    arg.derivation_path =
        Schema::Generic.derivation_path_ending_in(&ic_cdk::api::msg_caller(), arg.derivation_path);
    match ic_sign_with_ecdsa(&arg).await {
        Ok(result) => Ok(result),
        Err(sign_error) => {
            // Use a default rejection code and convert error to string
            Err(GenericSignWithEcdsaError::SigningError(
                RejectionCode::CanisterError,
                format!("{sign_error:?}")
            ))
        }
    }
}

/// The Schnorr public key issued by the Chain Fusion Signer to a given canister or user.
///
/// - To get your own public key, set `arg.canister_id` to `None`.
/// - To get the public key of another canister or user, set `arg.canister_id` to the principal of
///   the canister or user.
pub async fn schnorr_public_key(
    mut arg: SchnorrPublicKeyArgs,
) -> Result<SchnorrPublicKeyResult, SchnorrPublicKeyError> {
    // Moves the canister_id from the argument to the derivation path.
    let key_owner = arg.canister_id.take().unwrap_or_else(ic_cdk::api::msg_caller);
    if key_owner == Principal::anonymous() {
        ic_cdk::trap("Anonymous principal has no key.");
    }
    arg.derivation_path =
        Schema::Schnorr.derivation_path_ending_in(&key_owner, arg.derivation_path);
    debug_assert!(arg.canister_id.is_none());
    match ic_schnorr_public_key(&arg).await {
        Ok(result) => Ok(result),
        Err(call_error) => {
            // Use a default rejection code and convert error to string
            Err(SchnorrPublicKeyError::SigningError(
                RejectionCode::CanisterError,
                format!("{call_error:?}")
            ))
        }
    }
}

/// Sign with the Schnorr key issued by the Chain Fusion Signer to the caller.
pub async fn schnorr_sign(
    mut arg: SignWithSchnorrArgs,
) -> Result<SignWithSchnorrResult, SchnorrSigningError> {
    arg.derivation_path =
        Schema::Schnorr.derivation_path_ending_in(&ic_cdk::api::msg_caller(), arg.derivation_path);
    match ic_sign_with_schnorr(&arg).await {
        Ok(result) => Ok(result),
        Err(sign_error) => {
            // Use a default rejection code and convert error to string
            Err(SchnorrSigningError::SigningError(
                RejectionCode::CanisterError,
                format!("{sign_error:?}")
            ))
        }
    }
}
