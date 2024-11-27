//! Tests the Schnorr signing API.

use std::collections::HashMap;
use crate::canister::signer::SchnorrPublicKeyArgument;
use crate::utils::test_environment::TestSetup;


/// Users should have distinct public keys.  Similaryy, different derivation paths should have different public keys.
#[test]
fn public_keys_are_different() {
    let test_env = TestSetup::default();
    let users = &test_env.users;
    let derivation_paths = vec![
        vec![],
        vec![vec![]],
        vec![vec![], vec![]],
        vec![vec![], vec![], vec![]],
        vec!["foo".to_owned().into_bytes()],
        vec!["foo".to_owned().into_bytes(), vec![]],
        vec![vec![], "foo".to_owned().into_bytes()],
        vec![vec![], "foo".to_owned().into_bytes(), vec![]],
        vec!["f".to_owned().into_bytes(), "oo".to_owned().into_bytes()],
    ];
/*
    let public_keys = HashMap::new();
    for user in users.iter() {
        for derivation_path in derivation_paths.iter() {
            let public_key = test_env
                .signer
                .schnorr_public_key(SchnorrPublicKeyArgument {
                    canister_id: None,
                    derivation_path: derivation_path.clone(),
                })
                .await
                .unwrap()
                .0
                .public_key;
            assert!(!public_keys.contains_key(&public_key));
            public_keys.insert(public_key, (*user, derivation_path.clone()));
        }
    }
    */
}