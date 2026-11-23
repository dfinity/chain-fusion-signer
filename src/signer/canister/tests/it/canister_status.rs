//! Tests that only controllers can get the signer's canister status.
//!
//! `get_canister_status` takes no payment, but the signer pays for the management canister
//! call it makes.  Message inspection refuses ingress from anyone but a controller, so a
//! refused call is never inducted and costs the signer nothing.  Calls from other canisters
//! never pass through inspection, so the method's own guard has to refuse those.

use candid::{decode_one, encode_args, Principal};
use serde_bytes::ByteBuf;

use crate::{
    canister::signer::CanisterStatusResultV2,
    utils::{
        pic_canister::{cargo_wasm_path, PicCanister, PicCanisterBuilder, PicCanisterTrait},
        test_environment::TestSetup,
    },
};

/// The error the method's guard returns to a caller that is not a controller.
const NOT_A_CONTROLLER: &str = "Caller is not a controller.";

/// Callers that are not controllers of the signer.
fn non_controllers(test_env: &TestSetup) -> [Principal; 2] {
    [test_env.user, Principal::anonymous()]
}

/// Reads the signer's cycle balance.
fn signer_balance(test_env: &TestSetup) -> u128 {
    test_env.pic.cycle_balance(test_env.signer.canister_id)
}

/// Deploys the test proxy, a canister that forwards raw calls to another canister.
fn deploy_proxy(test_env: &TestSetup) -> PicCanister {
    PicCanisterBuilder::default()
        .with_wasm(&cargo_wasm_path("test_proxy"))
        .deploy_to(test_env.pic.clone())
}

/// Calls `get_canister_status` on the signer from the proxy canister.
///
/// Returns the raw reply, or the reason the signer rejected the call.
fn status_from_canister(test_env: &TestSetup, proxy: &PicCanister) -> Result<Vec<u8>, String> {
    proxy
        .update::<_, Result<ByteBuf, String>>(
            test_env.user,
            "call_raw",
            (
                test_env.signer.canister_id,
                "get_canister_status".to_string(),
                ByteBuf::from(encode_args(()).unwrap()),
            ),
        )
        .expect("Failed to call the proxy")
        .map(ByteBuf::into_vec)
}

#[test]
fn a_controller_gets_the_status() {
    let test_env = TestSetup::default();

    let status = test_env
        .signer
        .get_canister_status(test_env.controller)
        .expect("A controller should get the canister status");
    assert_eq!(
        status.settings.controllers,
        vec![test_env.controller],
        "The status should list the signer's controllers."
    );
}

/// Ingress from anyone else is refused by message inspection, before the method runs.
///
/// The guard's error never appears, which shows the method was not reached.  The anonymous
/// principal matters here because it controls a canister that `PocketIc` creates by default.
#[test]
fn non_controllers_are_refused_at_inspection() {
    let test_env = TestSetup::default();

    for caller in non_controllers(&test_env) {
        let error = test_env
            .pic
            .update_call(
                test_env.signer.canister_id,
                caller,
                "get_canister_status",
                encode_args(()).unwrap(),
            )
            .expect_err(&format!(
                "{caller} is not a controller and should be refused"
            ));
        assert!(
            !error.reject_message.contains(NOT_A_CONTROLLER),
            "Message inspection, not the method, should refuse {caller}, got: {error:?}"
        );
    }
}

/// A refused call costs the signer nothing, however often it is repeated.
#[test]
fn refused_ingress_costs_the_signer_nothing() {
    let test_env = TestSetup::default();

    let balance_before = signer_balance(&test_env);
    for caller in non_controllers(&test_env) {
        for _ in 0..10 {
            let status = test_env.signer.get_canister_status(caller);
            assert!(
                status.is_err(),
                "{caller} is not a controller and should be refused, got: {status:?}"
            );
        }
    }
    let spent = balance_before.saturating_sub(signer_balance(&test_env));
    assert_eq!(
        spent, 0,
        "Twenty refused calls should cost the signer nothing, but cost {spent} cycles."
    );

    // The same call from a controller does cost the signer, so the measurement above is not
    // vacuous.
    test_env
        .signer
        .get_canister_status(test_env.controller)
        .expect("A controller should get the canister status");
    let spent = balance_before.saturating_sub(signer_balance(&test_env));
    assert!(spent > 0, "A served call should cost the signer cycles.");
}

/// The guard refuses a canister that is not a controller.
///
/// Message inspection never sees a call from another canister, so this reaches the method.
#[test]
fn a_non_controller_canister_is_refused_by_the_guard() {
    let test_env = TestSetup::default();
    let proxy = deploy_proxy(&test_env);

    let error = status_from_canister(&test_env, &proxy)
        .expect_err("A canister that is not a controller should be refused");
    assert!(
        error.contains(NOT_A_CONTROLLER),
        "The method's guard should refuse the canister, got: {error}"
    );
}

/// The guard admits a canister that is a controller.
#[test]
fn a_controller_canister_gets_the_status() {
    let test_env = TestSetup::default();
    let proxy = deploy_proxy(&test_env);
    test_env
        .pic
        .set_controllers(
            test_env.signer.canister_id,
            Some(test_env.controller),
            vec![test_env.controller, proxy.canister_id],
        )
        .expect("Test setup error: Failed to add the proxy as a controller");

    let reply = status_from_canister(&test_env, &proxy)
        .expect("A canister that is a controller should get the canister status");
    let status: CanisterStatusResultV2 = decode_one(&reply).expect("Failed to decode the status");
    assert!(
        status.settings.controllers.contains(&proxy.canister_id),
        "The status should list the proxy as a controller, got: {status:?}"
    );
}
