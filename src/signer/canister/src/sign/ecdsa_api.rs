use ic_cdk::management_canister::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgs,
    SignWithEcdsaArgs,
};

use crate::state::read_config;

pub async fn get_ecdsa_signature(
    derivation_path: Vec<Vec<u8>>,
    message_hash: Vec<u8>,
) -> Result<Vec<u8>, String> {
    let key_name = read_config(|s| s.ecdsa_key_name.clone());
    let args = SignWithEcdsaArgs {
        message_hash,
        derivation_path,
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: key_name,
        },
    };

    let res = sign_with_ecdsa(&args)
        .await
        .map_err(|err| format!("{err:?}"))?;

    Ok(res.signature)
}

/// Computes the public key of the specified principal.
pub async fn ecdsa_pubkey_of(derivation_path: Vec<Vec<u8>>) -> Result<Vec<u8>, String> {
    let key_name = read_config(|s| s.ecdsa_key_name.clone());
    let args = EcdsaPublicKeyArgs {
        canister_id: None,
        derivation_path,
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: key_name,
        },
    };

    let response = ecdsa_public_key(&args)
        .await
        .map_err(|err| format!("{err:?}"))?;

    Ok(response.public_key)
}
