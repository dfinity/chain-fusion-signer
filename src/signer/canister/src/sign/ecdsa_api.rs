use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
    SignWithEcdsaArgument,
};

use crate::state::read_config;

pub async fn get_ecdsa_signature(
    derivation_path: Vec<Vec<u8>>,
    message_hash: Vec<u8>,
) -> Result<Vec<u8>, String> {
    let key_name = read_config(|s| s.ecdsa_key_name.clone());
    let key_id = EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: key_name,
    };

    let res = sign_with_ecdsa(SignWithEcdsaArgument {
        message_hash,
        derivation_path,
        key_id,
    })
    .await
    .map_err(|err| err.1)?;

    Ok(res.0.signature)
}

/// Computes the public key of the specified principal.
pub async fn ecdsa_pubkey_of(derivation_path: Vec<Vec<u8>>) -> Result<Vec<u8>, String> {
    let key_name = read_config(|s| s.ecdsa_key_name.clone());
    let response = ecdsa_public_key(EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path,
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: key_name,
        },
    })
    .await
    .map_err(|err| err.1)?;

    Ok(response.0.public_key)
}
