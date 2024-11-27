//! Tests the Schnorr signing API.

use std::collections::HashMap;
use serde_bytes::ByteBuf;

use crate::canister::signer::{SchnorrKeyId, SchnorrPublicKeyArgument, SchnorrAlgorithm, PaymentType};
use crate::utils::test_environment::TestSetup;


/// Users should have distinct public keys.  Similaryy, different derivation paths should have different public keys.
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
    ].into_iter().map(|paths| paths.into_iter().map(ByteBuf::from).collect()).collect();
    let mut public_keys = HashMap::new();
    for user in users.iter() {
        for derivation_path in derivation_paths.iter() {
            let public_key = test_env
                .signer
                .schnorr_public_key(*user, &SchnorrPublicKeyArgument {
                    key_id: SchnorrKeyId{
                        algorithm: SchnorrAlgorithm::Ed25519,
                        name: "test_key_1".to_string(),
                    },
                    canister_id: None,
                    derivation_path: derivation_path.clone(),
                }, &Some(PaymentType::AttachedCycles))
                .unwrap()
                .unwrap()
                .0
                .public_key;
            assert!(!public_keys.contains_key(&public_key), "These have the same public key: {:?} and {:?}", public_keys.get(&public_key), (user, derivation_path));
            public_keys.insert(public_key, (*user, derivation_path.clone()));
        }
    }
}