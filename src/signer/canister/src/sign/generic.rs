//! A generic signing API equivalent to that provided by the canister API.
use crate::derivation_path::Schema;
use ic_cdk::api::management_canister::{
    ecdsa::{
        self as canister_ecdsa, EcdsaPublicKeyArgument, EcdsaPublicKeyResponse,
        SignWithEcdsaArgument, SignWithEcdsaResponse,
    },
    schnorr::{
        SchnorrPublicKeyArgument, SchnorrPublicKeyResponse, SignWithSchnorrArgument,
        SignWithSchnorrResponse,
    },
};
use ic_chain_fusion_signer_api::types::generic::SignerSchnorrPublicKeyError;
pub use ic_chain_fusion_signer_api::types::generic::{
    GenericCallerEcdsaPublicKeyError, GenericSignWithEcdsaError,
};

/// A generic ECDSA public key for the user.
///
/// Warning: The user supplied derivation path is used as-is.  The caller is responsible for ensuring that unintended sub-keys are not requested.
pub async fn caller_ecdsa_public_key(
    mut arg: EcdsaPublicKeyArgument,
) -> Result<(EcdsaPublicKeyResponse,), GenericCallerEcdsaPublicKeyError> {
    arg.derivation_path =
        Schema::Generic.derivation_path_ending_in(&ic_cdk::caller(), arg.derivation_path);
    Ok(canister_ecdsa::ecdsa_public_key(arg).await?)
}

/// Signs a message with a generic ECDSA key for the user.
///
/// Warning: The user supplied derivation path is used as-is.  The caller is responsible for ensuring that unintended sub-keys are not requested.
pub async fn sign_with_ecdsa(
    mut arg: SignWithEcdsaArgument,
) -> Result<(SignWithEcdsaResponse,), GenericSignWithEcdsaError> {
    arg.derivation_path =
        Schema::Generic.derivation_path_ending_in(&ic_cdk::caller(), arg.derivation_path);
    Ok(canister_ecdsa::sign_with_ecdsa(arg).await?)
}

/// The Schnorr public key issued by the chain fusion signer to the caller or a specified canister or principal.
pub async fn schnorr_public_key(
    mut arg: SchnorrPublicKeyArgument,
) -> Result<(SchnorrPublicKeyResponse,), SignerSchnorrPublicKeyError> {
    // Moves the canister_id from the argument to the derivation path.
    let key_owner = arg.canister_id.take().unwrap_or_else(ic_cdk::caller);
    arg.derivation_path =
        Schema::Generic.derivation_path_ending_in(&key_owner, arg.derivation_path);
    debug_assert!(arg.canister_id.is_none());
    Ok(ic_cdk::api::management_canister::schnorr::schnorr_public_key(arg).await?)
}

/// Sign with Schnorr.
pub async fn schnorr_sign(
    mut arg: SignWithSchnorrArgument,
) -> Result<(SignWithSchnorrResponse,), ()> {
    arg.derivation_path =
        Schema::Generic.derivation_path_ending_in(&ic_cdk::caller(), arg.derivation_path);
    Ok(
        ic_cdk::api::management_canister::schnorr::sign_with_schnorr(arg)
            .await
            ?,
    )
}
