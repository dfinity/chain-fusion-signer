//! Tests that oversized ingress messages are rejected before they are inducted.
//!
//! The public key methods charge a flat fee, but the signer pays an ingress induction fee
//! proportional to the size of every message it accepts.  A caller who sends
//! maximum-size messages therefore makes the signer spend far more than it collects, even
//! though every call is valid, paid and successful.  Message inspection closes that gap by
//! refusing the message before the signer pays anything for it.
//!
//! The same filter bounds what a caller who never pays can cost: payment is taken inside the
//! method, after the signer has already paid to induct the message.

use candid::{encode_args, CandidType, Nat};
use ic_chain_fusion_signer_api::{
    limits::{MAX_MESSAGE_SIGNING_ARG_BYTES, MAX_PUBLIC_KEY_ARG_BYTES},
    methods::SignerMethods,
};
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
            self, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgs, EthPersonalSignRequest, PaymentType,
            SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgs, SignWithSchnorrArgs,
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

/// A `schnorr_sign` request whose message is `len` bytes long.
fn schnorr_sign_args(len: usize) -> SignWithSchnorrArgs {
    SignWithSchnorrArgs {
        aux: None,
        key_id: SchnorrKeyId {
            algorithm: SchnorrAlgorithm::Ed25519,
            name: KEY_NAME.to_string(),
        },
        derivation_path: vec![ByteBuf::from("MY APP")],
        message: ByteBuf::from(vec![0u8; len]),
    }
}

/// An `eth_personal_sign` request for a message of `len` bytes, hex encoded as callers send it.
fn personal_sign_request(len: usize) -> EthPersonalSignRequest {
    EthPersonalSignRequest {
        message: "00".repeat(len),
    }
}

/// The size, in bytes, of the Candid argument a call sends.
fn encoded_len<A: CandidType>(arg: &A, payment: &Option<PaymentType>) -> usize {
    encode_args((arg, payment))
        .expect("Failed to encode the arguments")
        .len()
}

/// The longest message whose request, built by `build`, encodes to at most `max_bytes`.
fn longest_within<A: CandidType>(
    max_bytes: usize,
    payment: &Option<PaymentType>,
    build: impl Fn(usize) -> A,
) -> usize {
    let size = |len: usize| encoded_len(&build(len), payment);
    // Start from the framing overhead, then step past any change in the length prefix.
    let mut len = max_bytes.saturating_sub(size(0));
    while size(len) > max_bytes {
        len -= 1;
    }
    while size(len + 1) <= max_bytes {
        len += 1;
    }
    len
}

/// The signer's response when message inspection refuses an ingress message.
const REFUSED_AT_INSPECTION: &str = "rejected the message";

/// Message signing requests up to the cap are accepted and signed.
///
/// Guards against the cap being set, or computed, below what a caller is told it can send.
#[test]
fn message_signing_up_to_the_ingress_limit_is_accepted() {
    let test_env = TestSetup::default();

    let payment = Some(approve(&test_env, SignerMethods::SchnorrSign, 1));
    let len = longest_within(MAX_MESSAGE_SIGNING_ARG_BYTES, &payment, schnorr_sign_args);
    let args = schnorr_sign_args(len);
    assert_eq!(
        encoded_len(&args, &payment),
        MAX_MESSAGE_SIGNING_ARG_BYTES,
        "The schnorr_sign request should be exactly at the limit."
    );
    let schnorr = test_env.signer.schnorr_sign(test_env.user, &args, &payment);
    assert!(
        matches!(schnorr, Ok(Ok(_))),
        "schnorr_sign should sign a {len} byte message at the ingress limit, got: {schnorr:?}"
    );

    let payment = Some(approve(&test_env, SignerMethods::EthPersonalSign, 1));
    let len = longest_within(
        MAX_MESSAGE_SIGNING_ARG_BYTES,
        &payment,
        personal_sign_request,
    );
    let request = personal_sign_request(len);
    // Hex takes two characters per byte, so the request can fall one byte short of the limit.
    assert!(
        encoded_len(&request, &payment) + 1 >= MAX_MESSAGE_SIGNING_ARG_BYTES,
        "The eth_personal_sign request should be at the limit."
    );
    let personal = test_env
        .signer
        .eth_personal_sign(test_env.user, &request, &payment);
    assert!(
        matches!(personal, Ok(Ok(_))),
        "eth_personal_sign should sign a {len} byte message at the ingress limit, got: \
         {personal:?}"
    );
}

/// A message signing request one message byte over the cap is refused by message inspection,
/// even from a caller who has approved payment, and costs the signer nothing.
#[test]
fn message_signing_over_the_ingress_limit_is_refused_at_inspection() {
    let test_env = TestSetup::default();
    // Approve enough for both calls, so that an accepted call could take payment and succeed.
    let payment = Some(approve(&test_env, SignerMethods::SchnorrSign, 2));
    let balance_before = signer_balance(&test_env);

    let len = longest_within(MAX_MESSAGE_SIGNING_ARG_BYTES, &payment, schnorr_sign_args) + 1;
    let args = schnorr_sign_args(len);
    assert_eq!(
        encoded_len(&args, &payment),
        MAX_MESSAGE_SIGNING_ARG_BYTES + 1,
        "The schnorr_sign request should be one byte over the limit."
    );
    let schnorr = test_env.signer.schnorr_sign(test_env.user, &args, &payment);
    assert!(
        matches!(&schnorr, Err(e) if e.contains(REFUSED_AT_INSPECTION)),
        "schnorr_sign should refuse a request over the ingress limit at inspection, got: \
         {schnorr:?}"
    );

    let len = longest_within(
        MAX_MESSAGE_SIGNING_ARG_BYTES,
        &payment,
        personal_sign_request,
    ) + 1;
    let request = personal_sign_request(len);
    assert!(
        encoded_len(&request, &payment) > MAX_MESSAGE_SIGNING_ARG_BYTES,
        "The eth_personal_sign request should be over the limit."
    );
    let personal = test_env
        .signer
        .eth_personal_sign(test_env.user, &request, &payment);
    assert!(
        matches!(&personal, Err(e) if e.contains(REFUSED_AT_INSPECTION)),
        "eth_personal_sign should refuse a request over the ingress limit at inspection, got: \
         {personal:?}"
    );

    let spent = balance_before.saturating_sub(signer_balance(&test_env));
    assert_eq!(
        spent, 0,
        "Refusing message signing requests over the limit should cost nothing, cost {spent} \
         cycles."
    );
}

/// Oversized message signing requests are refused before induction, also when the caller has
/// not paid.
///
/// Payment is taken after induction, so the ingress limit, not the fee, is what bounds the
/// induction cost of these methods.
#[test]
fn unfunded_oversized_message_signing_costs_nothing() {
    let test_env = TestSetup::default();
    // No approval and no attached cycles: an inducted call could not pay.
    let payment = None;

    let balance_before = signer_balance(&test_env);
    for _ in 0..3 {
        let schnorr = test_env.signer.schnorr_sign(
            test_env.user,
            &schnorr_sign_args(NEARLY_MAX_INGRESS),
            &payment,
        );
        assert!(
            matches!(&schnorr, Err(e) if e.contains(REFUSED_AT_INSPECTION)),
            "An oversized schnorr_sign request should be refused at inspection, got: {schnorr:?}"
        );

        let personal = test_env.signer.eth_personal_sign(
            test_env.user,
            &personal_sign_request(NEARLY_MAX_INGRESS / 2),
            &payment,
        );
        assert!(
            matches!(&personal, Err(e) if e.contains(REFUSED_AT_INSPECTION)),
            "An oversized eth_personal_sign request should be refused at inspection, got: \
             {personal:?}"
        );
    }
    let spent = balance_before.saturating_sub(signer_balance(&test_env));
    assert_eq!(
        spent, 0,
        "Refusing unfunded oversized message signing requests should cost nothing, cost {spent} \
         cycles."
    );
}
