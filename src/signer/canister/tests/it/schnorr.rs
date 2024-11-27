//! Tests the Schnorr signing API.

use std::collections::HashMap;
use ic_chain_fusion_signer_api::methods::SignerMethods;
use serde_bytes::ByteBuf;
use candid::Nat;

use crate::canister::signer::{self,SchnorrKeyId, SchnorrPublicKeyArgument, SchnorrAlgorithm, PaymentType};
use crate::canister::cycles_ledger::{self, ApproveArgs};
use crate::utils::test_environment::TestSetup;
use crate::utils::test_environment::LEDGER_FEE;

use ic_papi_api::principal2account;


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
        test_env
        .ledger
        .icrc_2_approve(test_env.user, &ApproveArgs::new(cycles_ledger::Account {
            owner: test_env.signer.canister_id,
            subaccount: Some(principal2account(&user)),
        }, Nat::from(SignerMethods::SchnorrPublicKey.fee() + LEDGER_FEE as u64)* derivation_paths.len() as u64))
        .expect("Failed to call ledger canister")
        .expect("Failed to approve payment");
        for derivation_path in derivation_paths.iter() {
            let public_key = test_env
                .signer
                .schnorr_public_key(*user, &SchnorrPublicKeyArgument {
                    key_id: SchnorrKeyId{
                        algorithm: SchnorrAlgorithm::Ed25519,
                        name: "dfx_test_key".to_string(),
                    },
                    canister_id: None,
                    derivation_path: derivation_path.clone(),
                }, &Some(PaymentType::PatronPaysIcrc2Cycles(signer::Account {
                    owner: test_env.user,
                    subaccount: None,
                })))
                .unwrap()
                .unwrap()
                .0
                .public_key;
            assert!(!public_keys.contains_key(&public_key), "These have the same public key: {:?} and {:?}", public_keys.get(&public_key), (user, derivation_path));
            public_keys.insert(public_key, (*user, derivation_path.clone()));
        }
    }
}