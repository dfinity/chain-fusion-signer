use std::str::FromStr;

use candid::Principal;
use ethers_core::{
    abi::ethereum_types::{Address, U256},
    types::transaction::eip2930::AccessList,
    utils::keccak256,
};
use ic_cdk::management_canister::{
    ecdsa_public_key, sign_with_ecdsa, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgs,
    SignWithEcdsaArgs,
};
pub use ic_chain_fusion_signer_api::types::eth::{
    EthAddressError, EthAddressRequest, EthAddressResponse,
};
use ic_chain_fusion_signer_api::types::{eth::EthSignTransactionError, transaction::SignRequest};
use k256::PublicKey;
use pretty_assertions::assert_eq;

use crate::{
    convert::{decode_hex, nat_to_u256, nat_to_u64},
    derivation_path::Schema,
    state::read_config,
};

/// Converts the public key bytes to an Ethereum address with a checksum.
pub fn pubkey_bytes_to_address(pubkey_bytes: &[u8]) -> String {
    use k256::elliptic_curve::sec1::ToEncodedPoint;

    let key =
        PublicKey::from_sec1_bytes(pubkey_bytes).expect("failed to parse the public key as SEC1");
    let point = key.to_encoded_point(false);
    // we re-encode the key to the decompressed representation.
    let point_bytes = point.as_bytes();
    assert_eq!(point_bytes[0], 0x04);

    let hash = keccak256(&point_bytes[1..]);

    ethers_core::utils::to_checksum(&Address::from_slice(&hash[12..32]), None)
}

/// Returns the public key and a message signature for the specified principal.
pub async fn pubkey_and_signature(caller: &Principal, message_hash: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let sign_args = SignWithEcdsaArgs {
        message_hash,
        derivation_path: Schema::Eth.derivation_path(caller),
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name: read_config(|s| s.ecdsa_key_name.clone()),
        },
    };

    // Fetch the pubkey and the signature concurrently to reduce latency.
    let (pubkey, response) = futures::join!(ecdsa_pubkey_of(caller), sign_with_ecdsa(&sign_args),);

    (
        pubkey,
        response.expect("failed to sign the message").signature,
    )
}

/// Computes the public key of the specified principal.
// TODO: Return a Result instead of panicking.
pub async fn ecdsa_pubkey_of(principal: &Principal) -> Vec<u8> {
    let name = read_config(|s| s.ecdsa_key_name.clone());
    let key = ecdsa_public_key(&EcdsaPublicKeyArgs {
        canister_id: None,
        derivation_path: Schema::Eth.derivation_path(principal),
        key_id: EcdsaKeyId {
            curve: EcdsaCurve::Secp256k1,
            name,
        },
    })
    .await
    .expect("failed to get public key");
    key.public_key
}

/// Computes the public key of the caller.
pub async fn eth_address(principal: Principal) -> Result<EthAddressResponse, EthAddressError> {
    Ok(EthAddressResponse {
        address: pubkey_bytes_to_address(&ecdsa_pubkey_of(&principal).await),
    })
}

/// Computes a signature for a precomputed hash.
pub async fn sign_prehash(prehash: String) -> String {
    let caller = ic_cdk::api::msg_caller();

    let hash_bytes = decode_hex(&prehash);

    let (pubkey, mut signature) = pubkey_and_signature(&caller, hash_bytes.to_vec()).await;

    let v = y_parity(&hash_bytes, &signature, &pubkey);
    signature.push(u8::try_from(v).unwrap_or_else(|_| {
        unreachable!("The value should be just one bit, so should fit easily into a byte")
    }));
    format!("0x{}", hex::encode(&signature))
}

/// Computes a signature for an [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559) transaction.
pub async fn sign_transaction(req: SignRequest) -> Result<String, EthSignTransactionError> {
    use ethers_core::types::{transaction::eip1559::Eip1559TransactionRequest, Signature};

    const EIP1559_TX_ID: u8 = 2;

    let caller = ic_cdk::api::msg_caller();

    let data = req.data.as_ref().map(|s| decode_hex(s));

    let tx = Eip1559TransactionRequest {
        chain_id: Some(nat_to_u64(&req.chain_id)),
        from: None,
        to: Some(
            Address::from_str(&req.to)
                .expect("failed to parse the destination address")
                .into(),
        ),
        gas: Some(nat_to_u256(&req.gas)),
        value: Some(nat_to_u256(&req.value)),
        nonce: Some(nat_to_u256(&req.nonce)),
        data,
        access_list: AccessList::default(),
        max_priority_fee_per_gas: Some(nat_to_u256(&req.max_priority_fee_per_gas)),
        max_fee_per_gas: Some(nat_to_u256(&req.max_fee_per_gas)),
    };

    let mut unsigned_tx_bytes = tx.rlp().to_vec();
    unsigned_tx_bytes.insert(0, EIP1559_TX_ID);

    let txhash = keccak256(&unsigned_tx_bytes);

    let (pubkey, signature) = pubkey_and_signature(&caller, txhash.to_vec()).await;

    let signature = Signature {
        v: y_parity(&txhash, &signature, &pubkey),
        r: U256::from_big_endian(&signature[0..32]),
        s: U256::from_big_endian(&signature[32..64]),
    };

    let mut signed_tx_bytes = tx.rlp_signed(&signature).to_vec();
    signed_tx_bytes.insert(0, EIP1559_TX_ID);

    Ok(format!("0x{}", hex::encode(&signed_tx_bytes)))
}

/// Computes a signature for a hex-encoded message according to [EIP-191](https://eips.ethereum.org/EIPS/eip-191).
pub async fn personal_sign(plaintext: String) -> String {
    let caller = ic_cdk::api::msg_caller();

    let bytes = decode_hex(&plaintext);

    let message = [
        b"\x19Ethereum Signed Message:\n",
        bytes.len().to_string().as_bytes(),
        bytes.as_ref(),
    ]
    .concat();

    let msg_hash = keccak256(&message);

    let (pubkey, mut signature) = pubkey_and_signature(&caller, msg_hash.to_vec()).await;

    let v = y_parity(&msg_hash, &signature, &pubkey);
    signature.push(u8::try_from(v).unwrap_or_else(|_| {
        unreachable!("The value should be one bit, so should easily fit into a byte")
    }));
    format!("0x{}", hex::encode(&signature))
}

/// Computes the parity bit allowing to recover the public key from the signature.
fn y_parity(prehash: &[u8], sig: &[u8], pubkey: &[u8]) -> u64 {
    use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

    let orig_key = VerifyingKey::from_sec1_bytes(pubkey).expect("failed to parse the pubkey");
    let signature = Signature::try_from(sig).unwrap();
    for parity in [0u8, 1] {
        let recid = RecoveryId::try_from(parity).unwrap();
        let recovered_key = VerifyingKey::recover_from_prehash(prehash, &signature, recid)
            .expect("failed to recover key");
        if recovered_key == orig_key {
            return u64::from(parity);
        }
    }

    panic!(
        "failed to recover the parity bit from a signature; sig: {}, pubkey: {}",
        hex::encode(sig),
        hex::encode(pubkey)
    )
}
