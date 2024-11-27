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
                    Nat::from(
                        SignerMethods::SchnorrPublicKey.fee()
                            + SignerMethods::SchnorrSign.fee()
                            + 2 * LEDGER_FEE as u64,
                    ) * derivation_paths.len() as u64,
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
                .public_key
                .into_vec();

            let signature = test_env
                .signer
                .schnorr_sign(
                    *user,
                    &signer::SignWithSchnorrArgument {
                        key_id: SchnorrKeyId {
                            algorithm: SchnorrAlgorithm::Ed25519,
                            name: "dfx_test_key".to_string(),
                        },
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

            // Sanity check
            assert_eq!(public_key.len(), 32, "Unexpected public key length");
            assert_eq!(signature.len(), 64, "Unexpected signature length");
            // Verify the signature
            {
                let signature = ed25519_dalek::Signature::try_from(signature.as_slice())
                    .expect("failed to deserialize signature");
                let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(
                    &<[u8; 32]>::try_from(public_key)
                        .expect("ed25519 public key should be 32 bytes"),
                )
                .expect("failed to sec1 deserialize public key");

                use ed25519_dalek::Verifier;
                let verification = verifying_key.verify(&message, &signature);
                assert!(verification.is_ok(), "Not verified: {:?}", verification);
            }
        }
    }
}
