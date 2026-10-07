//! Tests that message inspection refuses ingress from the anonymous principal.
//!
//! The paid methods reject the anonymous principal in their guards, but a guard runs only
//! after the signer has paid to induct the message.  Refusing the message during inspection
//! means it is never inducted, so it costs the signer nothing, whatever its method and size.

use candid::Principal;
use ic_chain_fusion_signer_api::limits::MAX_SMALL_ARG_BYTES;

use crate::utils::test_environment::TestSetup;

/// A payload just under the IC's ~2 MiB ingress ceiling, as in the ingress limit tests.
const NEARLY_MAX_INGRESS: usize = 1_900_000;

/// Every method that message inspection accepts ingress for.
const METHODS: [&str; 17] = [
    "schnorr_sign",
    "eth_personal_sign",
    "generic_caller_ecdsa_public_key",
    "generic_sign_with_ecdsa",
    "schnorr_public_key",
    "btc_caller_sign",
    "btc_caller_send",
    "eth_sign_transaction",
    "get_canister_status",
    "config",
    "http_request",
    "eth_address",
    "eth_address_of_caller",
    "eth_sign_prehash",
    "btc_caller_address",
    "btc_caller_balance",
    "btc_sign_prehash",
];

/// The methods that accept a message of any size.
const MESSAGE_SIGNING_METHODS: [&str; 2] = ["schnorr_sign", "eth_personal_sign"];

/// Reads the signer's cycle balance.
fn signer_balance(test_env: &TestSetup) -> u128 {
    test_env.pic.cycle_balance(test_env.signer.canister_id)
}

/// Ingress from the anonymous principal is refused for every method and costs the signer
/// nothing.
///
/// Each method gets an argument within its ingress limit, so message inspection would accept
/// it from any other caller; the message signing methods also get a nearly maximum-size one.
#[test]
fn anonymous_ingress_costs_the_signer_nothing() {
    let test_env = TestSetup::default();
    let small = vec![0u8; MAX_SMALL_ARG_BYTES];
    let large = vec![0u8; NEARLY_MAX_INGRESS];
    let messages = METHODS.iter().map(|method| (*method, &small)).chain(
        MESSAGE_SIGNING_METHODS
            .iter()
            .map(|method| (*method, &large)),
    );

    let balance_before = signer_balance(&test_env);
    for (method, arg) in messages {
        let response = test_env.pic.update_call(
            test_env.signer.canister_id,
            Principal::anonymous(),
            method,
            arg.clone(),
        );
        assert!(
            response.is_err(),
            "Anonymous ingress to {method} should be refused, got: {response:?}"
        );
    }
    let spent = balance_before.saturating_sub(signer_balance(&test_env));
    assert_eq!(
        spent, 0,
        "Refusing anonymous ingress should cost the signer nothing, but cost {spent} cycles."
    );
}

/// The same message from a principal that is not anonymous is inducted and paid for.
///
/// This shows that the balance check above would see the cost of an inducted message.
#[test]
fn ingress_from_a_user_is_inducted() {
    let test_env = TestSetup::default();

    let balance_before = signer_balance(&test_env);
    let response = test_env.pic.update_call(
        test_env.signer.canister_id,
        test_env.user,
        "schnorr_sign",
        vec![0u8; MAX_SMALL_ARG_BYTES],
    );
    assert!(
        response.is_err(),
        "A malformed argument should be rejected, got: {response:?}"
    );
    let spent = balance_before.saturating_sub(signer_balance(&test_env));
    assert!(
        spent > 0,
        "An inducted message should cost the signer cycles, but cost nothing."
    );
}
