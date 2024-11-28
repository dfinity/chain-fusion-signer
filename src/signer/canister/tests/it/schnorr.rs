//! Tests the Schnorr signing API.

use std::collections::HashMap;

use candid::{Nat, Principal};
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

/// The anonymous user should not be able to sign.
#[test]
fn anonymous_user_cannot_sign() {
    let test_env = TestSetup::default();
    let message = ByteBuf::from("pokemon");
    let signature = test_env.signer.schnorr_sign(
        Principal::anonymous(),
        &signer::SignWithSchnorrArgument {
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: "dfx_test_key".to_string(),
            },
            derivation_path: vec![],
            message: message.clone(),
        },
        &None,
    );
    assert_eq!(
        signature,
        Err("Anonymous caller not authorized.".to_string()),
        "The anonymous user should not be allowed to sign."
    );
}

/// It should not be possible to get the public key of the anonymous user.
#[test]
fn can_get_public_key_of_onymous_users_only() {
    let test_env = TestSetup::default();
    struct TestVector {
        user: Principal,
        can_get_public_key: bool,
    }
    let test_vectors = [
        TestVector {
            user: test_env.user,
            can_get_public_key: true,
        },
        TestVector {
            user: test_env.user2,
            can_get_public_key: true,
        },
        TestVector {
            user: Principal::anonymous(),
            can_get_public_key: false,
        },
    ];
    // Approve payment for the API calls with ICRC-2.
    test_env
        .ledger
        .icrc_2_approve(
            test_env.user,
            &ApproveArgs::new(
                cycles_ledger::Account {
                    owner: test_env.signer.canister_id,
                    subaccount: Some(principal2account(&test_env.user)),
                },
                Nat::from(SignerMethods::SchnorrPublicKey.fee() + LEDGER_FEE as u64)
                    * test_vectors.len(),
            ),
        )
        .expect("Failed to call ledger canister")
        .expect("Failed to approve payment");
    let payment_type = Some(PaymentType::PatronPaysIcrc2Cycles(signer::Account {
        owner: test_env.user,
        subaccount: None,
    }));
    // Check that only the expected public keys are available.
    for TestVector {
        user,
        can_get_public_key,
    } in test_vectors.iter()
    {
        let public_key = test_env.signer.schnorr_public_key(
            *user,
            &SchnorrPublicKeyArgument {
                key_id: SchnorrKeyId {
                    algorithm: SchnorrAlgorithm::Ed25519,
                    name: "dfx_test_key".to_string(),
                },
                canister_id: None,
                derivation_path: vec![],
            },
            &payment_type,
        );
        assert_eq!(
            public_key.is_ok(),
            *can_get_public_key,
            "Should {} get the public key of '{user}'.",
            if *can_get_public_key {
                "be able to"
            } else {
                "not be able to"
            }
        );
    }
}
// TODO: Verify that signing fails if not paid for, or if the payment is insufficient.

/// Users should have distinct public keys.  Similary, different derivation paths should have
/// different public keys.
#[test]
fn public_keys_are_different() {
    let test_env = TestSetup::default();
    // Variants to test:
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
    // TODO: Iterate over key types as well.

    // Each public key, together with the user and derivation path that produced it.
    // We will verify that no two user & derivation path pairs produce the same key.
    let mut public_keys = HashMap::new();
    // Loop over users and derivation paths...
    for user in users.iter() {
        // The `test_env.user` has funds and will act as patron for this user.
        // TODO: Add a method to `TestSetup` to allow:
        // ```
        // let payment_type = test_env.approve_payment_for_signer(test_env.user, amount);
        // ```
        // This would remove most of the payment code from tests like this for which payment isn't
        // the focus.
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
            // Verify that the public key is unique.
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
    // Variants to test:
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
    // Each user will get a public key and sign a message with it.  Payment is via ICRC-2 so each
    // trasaction also needs to pay the ledger fee.  This is how much that will cost:
    let cost_per_user = Nat::from({
        let num_tests = derivation_paths.len() * key_types.len();
        let cost_per_test = SignerMethods::SchnorrPublicKey.fee()
            + SignerMethods::SchnorrSign.fee()
            + 2 * LEDGER_FEE as u64;
        num_tests as u64 * cost_per_test
    });

    let message = ByteBuf::from("pokemon");
    for user in users.iter() {
        // Approve funds for the user.  `test_env.user` will act as patron.
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
        // Test all variants for that user.
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
///
/// The function signature is `(signature, public_key, message) -> Result<(), signature::Error>`.
fn schnorr_signature_verifier(
    algorithm: &SchnorrAlgorithm,
) -> impl Fn(&[u8], &[u8], &[u8]) -> signature::Result<()> {
    match algorithm {
        SchnorrAlgorithm::Bip340Secp256K1 => verify_schnorr_bip340_secp256k1_signature,
        SchnorrAlgorithm::Ed25519 => verify_schnorr_ed25519_signature,
    }
}

/// Verifies that a Schnorr Ed25519 signature is valid.
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

/// Verifies that a Schnorr secp256k1 signature is valid.
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
