//! Code for signing Bitcoin transactions.
use bitcoin::{Address, CompressedPublicKey, Network};
use candid::Principal;
use ic_cdk_bitcoin_canister::Network as BitcoinNetwork;
use ic_cdk_management_canister::{ecdsa_public_key, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgs};

use crate::{derivation_path::Schema, sign::ecdsa_api, state::read_config};

/// Computes the public key of the specified principal.
async fn ecdsa_pubkey_of(principal: &Principal) -> Result<Vec<u8>, String> {
    let name = read_config(|s| s.ecdsa_key_name.clone());
    if let Ok(key) = ecdsa_public_key(&EcdsaPublicKeyArgs {
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

pub fn transform_network(network: BitcoinNetwork) -> Network {
    match network {
        BitcoinNetwork::Mainnet => Network::Bitcoin,
        BitcoinNetwork::Testnet => Network::Testnet,
        BitcoinNetwork::Regtest => Network::Regtest,
    }
}

/// Signs a precomputed 32-byte digest under the caller's Bitcoin key (schema `Btc`).
///
/// Returns the raw 64-byte ECDSA signature (`r || s`). Unlike `btc_caller_sign`, which builds and
/// signs a transaction from supplied UTXOs, this signs an arbitrary digest, so the signature
/// verifies against the caller's P2WPKH address. This is what arbitrary message / PSBT signing
/// (e.g. `WalletConnect` `signMessage` / `signPsbt`) needs and `generic_sign_with_ecdsa` cannot
/// provide, as it derives a different (schema `Generic`) key.
pub async fn sign_prehash(principal: &Principal, message_hash: Vec<u8>) -> Result<Vec<u8>, String> {
    if message_hash.len() != 32 {
        return Err(format!(
            "expected a 32-byte digest, got {} bytes",
            message_hash.len()
        ));
    }
    ecdsa_api::get_ecdsa_signature(Schema::Btc.derivation_path(principal), message_hash).await
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
