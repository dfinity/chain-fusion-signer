//! Code for signing Bitcoin transactions.
use crate::derivation_path::Schema;
use crate::state::read_config;
use bitcoin::{Address, CompressedPublicKey, Network};
use candid::Principal;
use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
};

/// Computes the public key of the specified principal.
async fn ecdsa_pubkey_of(principal: &Principal) -> Result<Vec<u8>, String> {
    let name = read_config(|s| s.ecdsa_key_name.clone());
    if let Ok((key,)) = ecdsa_public_key(EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: Schema::Btc.derivation_path(principal),
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name,
        },
    })
    .await
    {
        Ok(key.public_key)
    } else {
        Err("Failed to get ecdsa public key".to_string())
    }
}

fn transform_network(network: BitcoinNetwork) -> Network {
    match network {
        BitcoinNetwork::Mainnet => Network::Bitcoin,
        BitcoinNetwork::Testnet => Network::Testnet,
        BitcoinNetwork::Regtest => Network::Regtest,
    }
}

/// Converts a public key to a P2PKH address.
pub async fn principal_to_p2wpkh_address(
    network: BitcoinNetwork,
    principal: &Principal,
) -> Result<String, String> {
    let ecdsa_pubkey = ecdsa_pubkey_of(principal)
        .await
        .map_err(|_| "Error getting ECDSA public key".to_string())?;
    if let Ok(compressed_public_key) = CompressedPublicKey::from_slice(&ecdsa_pubkey) {
        Ok(Address::p2wpkh(&compressed_public_key, transform_network(network)).to_string())
    } else {
        Err("Error getting P2WPKH from public key".to_string())
    }
}
