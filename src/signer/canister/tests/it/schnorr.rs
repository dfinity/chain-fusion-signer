//! Tests the Schnorr signing API.

use std::collections::HashMap;

use candid::Nat;
use ic_chain_fusion_signer_api::methods::SignerMethods;
use ic_papi_api::principal2account;
use serde_bytes::ByteBuf;

use crate::{
    canister::{
        cycles_ledger::{self, ApproveArgs},
        signer::{self, PaymentType, SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgument},
    },
    utils::test_environment::{TestSetup, LEDGER_FEE},
};

/// Users should have distinct public keys.  Similaryy, different derivation paths should have
/// different public keys.
#[test]
fn public_keys_are_different() {
    let test_env = TestSetup::default();
    let users = &test_env.users;
    let derivation_paths: Vec<Vec<ByteBuf>> = [
        vec![],
        vec![""],
        vec!["", ""],
        vec!["", "", ""],
        vec!["foo"],
        vec!["foo", ""],
        vec!["", "foo"],
        vec!["", "foo", ""],
        vec!["f", "oo"],
        ["flummoxed"; 244].to_vec(),
    ]
    .into_iter()
    .map(|paths| paths.into_iter().map(ByteBuf::from).collect())
    .collect();
    let mut public_keys = HashMap::new();
    for user in users.iter() {
        test_env
            .ledger
            .icrc_2_approve(
                test_env.user,
                &ApproveArgs::new(
                    cycles_ledger::Account {
                        owner: test_env.signer.canister_id,
                        subaccount: Some(principal2account(&user)),
                    },
                    Nat::from(SignerMethods::SchnorrPublicKey.fee() + LEDGER_FEE as u64)
                        * derivation_paths.len() as u64,
                ),
            )
            .expect("Failed to call ledger canister")
            .expect("Failed to approve payment");
        for derivation_path in derivation_paths.iter() {
            let public_key = test_env
                .signer
                .schnorr_public_key(
                    *user,
                    &SchnorrPublicKeyArgument {
                        key_id: SchnorrKeyId {
                            algorithm: SchnorrAlgorithm::Ed25519,
                            name: "dfx_test_key".to_string(),
                        },
                        canister_id: None,
                        derivation_path: derivation_path.clone(),
                    },
                    &Some(PaymentType::PatronPaysIcrc2Cycles(signer::Account {
                        owner: test_env.user,
                        subaccount: None,
                    })),
                )
                .unwrap()
                .unwrap()
                .0
                .public_key;
            assert!(
                !public_keys.contains_key(&public_key),
                "These have the same public key: {:?} and {:?}",
                public_keys.get(&public_key),
                (user, derivation_path)
            );
            public_keys.insert(public_key, (*user, derivation_path.clone()));
        }
    }
}

/// Signatures should be verifiable with the corresponding public key.
#[test]
fn signatures_can_be_verified() {
    let test_env = TestSetup::default();
    let users = &test_env.users;
    let derivation_paths: Vec<Vec<ByteBuf>> = [vec![], vec!["", "", ""], vec!["foo"]]
        .into_iter()
        .map(|paths| paths.into_iter().map(ByteBuf::from).collect())
        .collect();
    let key_types = [
        SchnorrKeyId {
            algorithm: SchnorrAlgorithm::Ed25519,
            name: "dfx_test_key".to_string(),
        },
        SchnorrKeyId {
            algorithm: SchnorrAlgorithm::Bip340Secp256K1,
            name: "dfx_test_key".to_string(),
        },
    ];
    let cost_per_user = Nat::from({
        let num_tests = derivation_paths.len() * key_types.len();
        let cost_per_test = SignerMethods::SchnorrPublicKey.fee()
            + SignerMethods::SchnorrSign.fee()
            + 2 * LEDGER_FEE as u64;
        num_tests as u64 * cost_per_test
    });

    let message = ByteBuf::from("pokemon");
    for user in users.iter() {
        test_env
            .ledger
            .icrc_2_approve(
                test_env.user,
                &ApproveArgs::new(
                    cycles_ledger::Account {
                        owner: test_env.signer.canister_id,
                        subaccount: Some(principal2account(&user)),
                    },
                    cost_per_user.clone(),
                ),
            )
            .expect("Failed to call ledger canister")
            .expect("Failed to approve payment");
        for key_type in key_types.iter() {
            for derivation_path in derivation_paths.iter() {
                let public_key = test_env
                    .signer
                    .schnorr_public_key(
                        *user,
                        &SchnorrPublicKeyArgument {
                            key_id: key_type.clone(),
                            canister_id: None,
                            derivation_path: derivation_path.clone(),
                        },
                        &Some(PaymentType::PatronPaysIcrc2Cycles(signer::Account {
                            owner: test_env.user,
                            subaccount: None,
                        })),
                    )
                    .unwrap()
                    .unwrap()
                    .0
                    .public_key
                    .into_vec();

                let signature = test_env
                    .signer
                    .schnorr_sign(
                        *user,
                        &signer::SignWithSchnorrArgument {
                            key_id: key_type.clone(),
                            derivation_path: derivation_path.clone(),
                            message: message.clone(),
                        },
                        &Some(PaymentType::PatronPaysIcrc2Cycles(signer::Account {
                            owner: test_env.user,
                            subaccount: None,
                        })),
                    )
                    .unwrap()
                    .unwrap()
                    .0
                    .signature
                    .into_vec();

                // Verify the signature
                let verification = schnorr_signature_verifier(&key_type.algorithm)(
                    &signature,
                    &public_key,
                    &message,
                );
                assert!(
                    verification.is_ok(),
                    "Failed to verify signature: {verification:?}\nalgorithm: {:?}",
                    key_type.algorithm
                );
            }
        }
    }
}

/// The verification function for a given type of Schnorr key.
fn schnorr_signature_verifier(
    algorithm: &SchnorrAlgorithm,
) -> impl Fn(&[u8], &[u8], &[u8]) -> signature::Result<()> {
    match algorithm {
        SchnorrAlgorithm::Bip340Secp256K1 => verify_schnorr_bip340_secp256k1_signature,
        SchnorrAlgorithm::Ed25519 => verify_schnorr_ed25519_signature,
    }
}

fn verify_schnorr_ed25519_signature(
    signature_bytes: &[u8],
    public_key_bytes: &[u8],
    message_bytes: &[u8],
) -> signature::Result<()> {
    // Source: https://github.com/dfinity/chainkey-testing-canister/blob/ed37234d41d4c528b86bbe158c80c9277f4fd17c/tests/tests.rs#L104
    let signature = ed25519_dalek::Signature::try_from(signature_bytes)?;
    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(
        &<[u8; 32]>::try_from(public_key_bytes).map_err(|_| signature::Error::new())?,
    )?;
    use ed25519_dalek::Verifier;
    verifying_key.verify(message_bytes, &signature)
}

fn verify_schnorr_bip340_secp256k1_signature(
    signature_bytes: &[u8],
    public_key_bytes: &[u8],
    message_bytes: &[u8],
) -> signature::Result<()> {
    // Source: https://github.com/dfinity/chainkey-testing-canister/blob/ed37234d41d4c528b86bbe158c80c9277f4fd17c/tests/tests.rs#L69
    let signature = k256::schnorr::Signature::try_from(signature_bytes)?;
    let verifying_key = k256::schnorr::VerifyingKey::from_bytes(&public_key_bytes[1..])?;
    verifying_key.verify_raw(&message_bytes, &signature)
}
