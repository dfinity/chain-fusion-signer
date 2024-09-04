//! Code for signing Bitcoin transactions.
use crate::derivation_path;
use crate::state::read_config;
use candid::Principal;
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
};

/// Computes the public key of the specified principal.
pub async fn ecdsa_pubkey_of(principal: &Principal) -> Vec<u8> {
    let name = read_config(|s| s.ecdsa_key_name.clone());
    let (key,) = ecdsa_public_key(EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: derivation_path::btc(principal),
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name,
        },
    })
    .await
    .expect("failed to get public key");
    key.public_key
}
