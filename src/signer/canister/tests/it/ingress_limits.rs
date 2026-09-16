//! Tests that oversized ingress messages are rejected before they are inducted.
//!
//! The public key methods charge a flat fee, but the signer pays an ingress induction fee
//! proportional to the size of every message it accepts.  A caller who sends
//! maximum-size messages therefore makes the signer spend far more than it collects, even
//! though every call is valid, paid and successful.  Message inspection closes that gap by
//! refusing the message before the signer pays anything for it.

use candid::Nat;
use ic_chain_fusion_signer_api::{limits::MAX_PUBLIC_KEY_ARG_BYTES, methods::SignerMethods};
use ic_papi_api::principal2account;
use serde_bytes::ByteBuf;

/// A payload just under the IC's ~2 MiB ingress ceiling.
///
/// The ceiling applies to the whole signed message, so this leaves room for the envelope;
/// anything larger is refused by the network before it reaches the canister at all, which
/// is not what these tests are about.
const NEARLY_MAX_INGRESS: usize = 1_900_000;

use crate::{
    canister::{
        cycles_ledger::{self, ApproveArgs},
        signer::{
            self, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgs, PaymentType, SchnorrAlgorithm,
            SchnorrKeyId, SchnorrPublicKeyArgs,
        },
    },
    utils::test_environment::{TestSetup, LEDGER_FEE},
};

/// The key name deployed by [`TestSetup`].
const KEY_NAME: &str = "test_key_1";

/// A derivation path that is well over the ingress limit.
///
/// One element holding most of an ingress message is the shape used by the reported cycle
/// drain; the real attack goes up to the ~2 MiB ingress limit, and anything over
/// [`MAX_PUBLIC_KEY_ARG_BYTES`] is refused the same way.
fn oversized_path() -> Vec<ByteBuf> {
    vec![ByteBuf::from(vec![0u8; MAX_PUBLIC_KEY_ARG_BYTES + 1])]
}

/// Approves enough cycles for `calls` calls to `method`, so that an accepted call would be
/// able to take payment.
fn approve(test_env: &TestSetup, method: SignerMethods, calls: u128) -> PaymentType {
    test_env
        .ledger
        .icrc2_approve(
            test_env.user,
            &ApproveArgs::new(
                cycles_ledger::Account {
                    owner: test_env.signer.canister_id,
                    subaccount: Some(principal2account(&test_env.user)),
                },
                Nat::from((method.fee() + LEDGER_FEE) * calls),
            ),
        )
        .expect("Failed to call ledger canister")
        .expect("Failed to approve payment");
    PaymentType::PatronPaysIcrc2Cycles(signer::Account {
        owner: test_env.user,
        subaccount: None,
    })
}

/// Reads the signer's cycle balance.
fn signer_balance(test_env: &TestSetup) -> u128 {
    test_env.pic.cycle_balance(test_env.signer.canister_id)
}

/// An oversized ingress message never reaches either public key method.
#[test]
fn oversized_ingress_is_refused() {
    let test_env = TestSetup::default();
    let payment = Some(approve(&test_env, SignerMethods::SchnorrPublicKey, 2));

    let schnorr = test_env.signer.schnorr_public_key(
        test_env.user,
        &SchnorrPublicKeyArgs {
            canister_id: None,
            derivation_path: oversized_path(),
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: KEY_NAME.to_string(),
            },
        },
        &payment,
    );
    assert!(
        schnorr.is_err(),
        "schnorr_public_key should refuse an oversized ingress message, got: {schnorr:?}"
    );

    let ecdsa = test_env.signer.generic_caller_ecdsa_public_key(
        test_env.user,
        &EcdsaPublicKeyArgs {
            canister_id: None,
            derivation_path: oversized_path(),
            key_id: EcdsaKeyId {
                curve: EcdsaCurve::Secp256k1,
                name: KEY_NAME.to_string(),
            },
        },
        &payment,
    );
    assert!(
        ecdsa.is_err(),
        "generic_caller_ecdsa_public_key should refuse an oversized ingress message, got: {ecdsa:?}"
    );
}

/// A refused message costs the signer nothing.
///
/// This is the property that closes the reported amplification: the attack is only worth
/// mounting because each call leaves the signer poorer than the fee it collects.  A
/// message that is never inducted carries no induction fee and no execution, so repeating
/// it cannot drain the signer.
#[test]
fn refused_ingress_costs_the_signer_nothing() {
    let test_env = TestSetup::default();
    let payment = Some(approve(&test_env, SignerMethods::SchnorrPublicKey, 20));
    let key_id = SchnorrKeyId {
        algorithm: SchnorrAlgorithm::Ed25519,
        name: KEY_NAME.to_string(),
    };

    let balance_before = signer_balance(&test_env);
    for _ in 0..20 {
        let response = test_env.signer.schnorr_public_key(
            test_env.user,
            &SchnorrPublicKeyArgs {
                canister_id: None,
                derivation_path: oversized_path(),
                key_id: key_id.clone(),
            },
            &payment,
        );
        assert!(
            response.is_err(),
            "Every oversized ingress message should be refused, got: {response:?}"
        );
    }
    let spent = balance_before.saturating_sub(signer_balance(&test_env));
    assert_eq!(
        spent, 0,
        "Twenty refused ingress messages should cost the signer nothing, but cost {spent} cycles."
    );
}

/// Ordinary requests are unaffected: message inspection has to accept everything else.
///
/// Defining `canister_inspect_message` makes the canister reject every ingress update call
/// it does not explicitly accept, so this guards against the filter silently disabling the
/// API.
#[test]
fn ordinary_ingress_is_accepted() {
    let test_env = TestSetup::default();
    let payment = Some(approve(&test_env, SignerMethods::SchnorrPublicKey, 1));

    let public_key = test_env.signer.schnorr_public_key(
        test_env.user,
        &SchnorrPublicKeyArgs {
            canister_id: None,
            derivation_path: vec![ByteBuf::from("MY APP")],
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: KEY_NAME.to_string(),
            },
        },
        &payment,
    );
    assert!(
        matches!(public_key, Ok(Ok(_))),
        "An ordinary request should still return a public key, got: {public_key:?}"
    );

    // A method in a different limit tier still works too.
    let status = test_env.signer.get_canister_status(test_env.user);
    assert!(
        status.is_ok(),
        "An ordinary call to a method with a smaller ingress limit should still be accepted, \
         got: {status:?}"
    );
}

/// An oversized message to an unpaid, zero-argument method costs the signer nothing.
///
/// `get_canister_status` takes no arguments and collects no fee, so before every method had
/// a limit a caller could send it a maximum-size blob, have dispatch reject the malformed
/// argument, and leave the signer paying induction for it — a cycle drain needing no
/// funding at all.
#[test]
fn oversized_ingress_to_an_unpaid_method_costs_nothing() {
    let test_env = TestSetup::default();
    let blob = vec![0u8; NEARLY_MAX_INGRESS];

    let balance_before = signer_balance(&test_env);
    for _ in 0..5 {
        let response = test_env.pic.update_call(
            test_env.signer.canister_id,
            test_env.user,
            "get_canister_status",
            blob.clone(),
        );
        assert!(
            response.is_err(),
            "An oversized message to get_canister_status should be refused, got: {response:?}"
        );
    }
    let spent = balance_before.saturating_sub(signer_balance(&test_env));
    assert_eq!(
        spent, 0,
        "Refusing oversized messages to an unpaid method should cost nothing, cost {spent} cycles."
    );
}

/// A call to a method that does not exist is refused before it is inducted.
///
/// Dispatch would reject it anyway; refusing it here means the signer does not pay to
/// induct a maximum-size message first.
#[test]
fn unknown_methods_are_refused_for_free() {
    let test_env = TestSetup::default();

    let balance_before = signer_balance(&test_env);
    let response = test_env.pic.update_call(
        test_env.signer.canister_id,
        test_env.user,
        "not_a_method",
        vec![0u8; NEARLY_MAX_INGRESS],
    );
    assert!(
        response.is_err(),
        "An unknown method should be refused, got: {response:?}"
    );
    let spent = balance_before.saturating_sub(signer_balance(&test_env));
    assert_eq!(spent, 0, "Refusing an unknown method cost {spent} cycles.");
}
