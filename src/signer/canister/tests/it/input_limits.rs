//! Tests that oversized caller-supplied inputs are rejected.
//!
//! The public key methods charge a flat fee but pay costs that grow with the number of
//! caller-supplied bytes, so an unbounded derivation path lets a funded caller drain the
//! signer's cycle balance and take the shared service offline.  These tests pin down the
//! two properties that prevent it: oversized input is refused, and refusing it neither
//! charges the caller nor costs the signer.

use candid::Nat;
use ic_chain_fusion_signer_api::{
    limits::{MAX_DERIVATION_PATH_BYTES, MAX_DERIVATION_PATH_ELEMENTS, MAX_KEY_NAME_BYTES},
    methods::SignerMethods,
};
use ic_papi_api::principal2account;
use serde_bytes::ByteBuf;

use crate::{
    canister::{
        cycles_ledger::{self, ApproveArgs},
        signer::{
            self, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgs, GenericCallerEcdsaPublicKeyError,
            GenericSignWithEcdsaError, PaymentType, SchnorrAlgorithm, SchnorrKeyId,
            SchnorrPublicKeyArgs, SchnorrPublicKeyError, SchnorrSigningError, SignWithEcdsaArgs,
            SignWithSchnorrArgs,
        },
    },
    utils::test_environment::{TestSetup, LEDGER_FEE},
};

/// The key name deployed by [`TestSetup`].
const KEY_NAME: &str = "test_key_1";

/// A derivation path that is one byte over the limit, in a single element.
///
/// This is the shape used by the reported cycle drain: one component holding almost the
/// whole ingress message.
fn oversized_path() -> Vec<ByteBuf> {
    vec![ByteBuf::from(vec![0u8; MAX_DERIVATION_PATH_BYTES + 1])]
}

/// The largest derivation path a caller is allowed to send.
fn maximal_path() -> Vec<ByteBuf> {
    vec![ByteBuf::from(vec![0u8; MAX_DERIVATION_PATH_BYTES])]
}

fn ecdsa_key_id() -> EcdsaKeyId {
    EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        name: KEY_NAME.to_string(),
    }
}

fn schnorr_key_id() -> SchnorrKeyId {
    SchnorrKeyId {
        algorithm: SchnorrAlgorithm::Ed25519,
        name: KEY_NAME.to_string(),
    }
}

/// Approves enough cycles for `calls` calls to `method`, so that a rejected call would be
/// able to take payment if it tried to.
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

/// An oversized derivation path is rejected by every method that forwards one.
#[test]
fn oversized_derivation_path_is_rejected() {
    let test_env = TestSetup::default();
    let user = test_env.user;
    // One approval large enough for all four calls, so that a missing rejection would show
    // up as a successful call rather than as a payment failure.
    let payment = Some(approve(&test_env, SignerMethods::GenericSignWithEcdsa, 4));

    let ecdsa_public_key = test_env.signer.generic_caller_ecdsa_public_key(
        user,
        &EcdsaPublicKeyArgs {
            canister_id: None,
            derivation_path: oversized_path(),
            key_id: ecdsa_key_id(),
        },
        &payment,
    );
    assert!(
        matches!(
            ecdsa_public_key,
            Ok(Err(GenericCallerEcdsaPublicKeyError::InvalidArgument { .. }))
        ),
        "generic_caller_ecdsa_public_key should reject an oversized derivation path, got: {ecdsa_public_key:?}"
    );

    let ecdsa_signature = test_env.signer.generic_sign_with_ecdsa(
        user,
        &payment,
        &SignWithEcdsaArgs {
            message_hash: ByteBuf::from(vec![0u8; 32]),
            derivation_path: oversized_path(),
            key_id: ecdsa_key_id(),
        },
    );
    assert!(
        matches!(
            ecdsa_signature,
            Ok(Err(GenericSignWithEcdsaError::InvalidArgument { .. }))
        ),
        "generic_sign_with_ecdsa should reject an oversized derivation path, got: {ecdsa_signature:?}"
    );

    let schnorr_public_key = test_env.signer.schnorr_public_key(
        user,
        &SchnorrPublicKeyArgs {
            canister_id: None,
            derivation_path: oversized_path(),
            key_id: schnorr_key_id(),
        },
        &payment,
    );
    assert!(
        matches!(
            schnorr_public_key,
            Ok(Err(SchnorrPublicKeyError::InvalidArgument { .. }))
        ),
        "schnorr_public_key should reject an oversized derivation path, got: {schnorr_public_key:?}"
    );

    let schnorr_signature = test_env.signer.schnorr_sign(
        user,
        &SignWithSchnorrArgs {
            aux: None,
            key_id: schnorr_key_id(),
            derivation_path: oversized_path(),
            message: ByteBuf::from("pokemon"),
        },
        &payment,
    );
    assert!(
        matches!(
            schnorr_signature,
            Ok(Err(SchnorrSigningError::InvalidArgument { .. }))
        ),
        "schnorr_sign should reject an oversized derivation path, got: {schnorr_signature:?}"
    );
}

/// Too many derivation path elements are rejected, even when they are empty.
///
/// Counting bytes alone would not catch this, and the management canister would reject the
/// path only after the signer had paid to transmit it.
#[test]
fn too_many_derivation_path_elements_are_rejected() {
    let test_env = TestSetup::default();
    let payment = Some(approve(&test_env, SignerMethods::SchnorrPublicKey, 1));

    let response = test_env.signer.schnorr_public_key(
        test_env.user,
        &SchnorrPublicKeyArgs {
            canister_id: None,
            derivation_path: vec![ByteBuf::new(); MAX_DERIVATION_PATH_ELEMENTS + 1],
            key_id: schnorr_key_id(),
        },
        &payment,
    );
    assert!(
        matches!(
            response,
            Ok(Err(SchnorrPublicKeyError::InvalidArgument { .. }))
        ),
        "schnorr_public_key should reject too many derivation path elements, got: {response:?}"
    );
}

/// An oversized key name is rejected: the signer forwards it verbatim and pays for it.
#[test]
fn oversized_key_name_is_rejected() {
    let test_env = TestSetup::default();
    let payment = Some(approve(&test_env, SignerMethods::SchnorrPublicKey, 1));

    let response = test_env.signer.schnorr_public_key(
        test_env.user,
        &SchnorrPublicKeyArgs {
            canister_id: None,
            derivation_path: vec![],
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: "k".repeat(MAX_KEY_NAME_BYTES + 1),
            },
        },
        &payment,
    );
    assert!(
        matches!(
            response,
            Ok(Err(SchnorrPublicKeyError::InvalidArgument { .. }))
        ),
        "schnorr_public_key should reject an oversized key name, got: {response:?}"
    );
}

/// A derivation path exactly at the limit still returns a public key.
///
/// The cap has to leave legitimate callers alone; 4 KiB is far above the documented
/// application-name and BIP32-style paths.
#[test]
fn maximal_derivation_path_is_accepted() {
    let test_env = TestSetup::default();
    let payment = Some(approve(&test_env, SignerMethods::SchnorrPublicKey, 1));

    let response = test_env.signer.schnorr_public_key(
        test_env.user,
        &SchnorrPublicKeyArgs {
            canister_id: None,
            derivation_path: maximal_path(),
            key_id: schnorr_key_id(),
        },
        &payment,
    );
    assert!(
        matches!(response, Ok(Ok(_))),
        "schnorr_public_key should accept a derivation path at the limit, got: {response:?}"
    );
}

/// Rejecting an oversized request does not charge the caller, and costs the signer less
/// than serving a legitimate one.
///
/// This is the property that closes the reported amplification.  The signer still pays to
/// induct and execute a rejected ingress message — every canister does — but the cost is
/// now bounded by the ingress limit instead of growing with the caller's derivation path,
/// so a rejected request can never cost more than the fee an accepted one pays.
#[test]
fn rejected_requests_do_not_charge_the_caller_and_stay_cheap() {
    let test_env = TestSetup::default();
    let user = test_env.user;
    const CALLS: u128 = 10;
    // Approve enough for every call, so that a missing rejection would show up as a
    // successful call rather than as a payment failure.
    let payment = Some(approve(&test_env, SignerMethods::SchnorrPublicKey, CALLS));
    let allowance_args = cycles_ledger::AllowanceArgs {
        account: cycles_ledger::Account {
            owner: user,
            subaccount: None,
        },
        spender: cycles_ledger::Account {
            owner: test_env.signer.canister_id,
            subaccount: Some(principal2account(&user)),
        },
    };

    let allowance_before = test_env
        .ledger
        .icrc2_allowance(user, &allowance_args)
        .expect("Failed to get the allowance");
    let balance_before = test_env.pic.cycle_balance(test_env.signer.canister_id);

    for _ in 0..CALLS {
        let response = test_env.signer.schnorr_public_key(
            user,
            &SchnorrPublicKeyArgs {
                canister_id: None,
                derivation_path: oversized_path(),
                key_id: schnorr_key_id(),
            },
            &payment,
        );
        assert!(
            matches!(
                response,
                Ok(Err(SchnorrPublicKeyError::InvalidArgument { .. }))
            ),
            "Every oversized request should be rejected, got: {response:?}"
        );
    }

    let allowance_after = test_env
        .ledger
        .icrc2_allowance(user, &allowance_args)
        .expect("Failed to get the allowance");
    assert_eq!(
        allowance_before.allowance, allowance_after.allowance,
        "A rejected request must not charge the caller."
    );

    let fee = SignerMethods::SchnorrPublicKey.fee();
    let spent =
        balance_before.saturating_sub(test_env.pic.cycle_balance(test_env.signer.canister_id));
    let spent_per_call = spent / CALLS;
    assert!(
        spent_per_call < fee,
        "A rejected request cost the signer {spent_per_call} cycles, which is more than the \
         {fee} cycle fee it charges for serving one."
    );
}
