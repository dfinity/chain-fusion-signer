//! Code for signing Bitcoin transactions.
use crate::derivation_path::Schema;
use crate::state::read_config;
use bitcoin::{Address, Network, PublicKey};
use candid::Principal;
use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
};

/// Computes the public key of the specified principal.
pub async fn ecdsa_pubkey_of(principal: &Principal) -> Vec<u8> {
    let name = read_config(|s| s.ecdsa_key_name.clone());
    let (key,) = ecdsa_public_key(EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: Schema::Btc.derivation_path(principal),
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name,
        },
    })
    .await
    .expect("failed to get public key");
    key.public_key
}

fn transform_network(network: BitcoinNetwork) -> Network {
    match network {
        BitcoinNetwork::Mainnet => Network::Bitcoin,
        BitcoinNetwork::Testnet => Network::Testnet,
        BitcoinNetwork::Regtest => Network::Regtest,
    }
}

/// Converts a public key to a P2PKH address.
/// Reference: [IC Bitcoin Documentation](https://internetcomputer.org/docs/current/developer-docs/multi-chain/bitcoin/using-btc/generate-addresses#generating-addresses-with-threshold-ecdsa)
pub fn public_key_to_p2pkh_address(network: BitcoinNetwork, public_key: &[u8]) -> String {
    Address::p2pkh(
        PublicKey::from_slice(public_key).expect("failed to parse public key"),
        transform_network(network),
    )
    .to_string()
}
