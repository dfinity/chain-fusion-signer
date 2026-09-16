//! A test-only canister that forwards a raw call to another canister.
//!
//! Integration tests use it to reach a canister's methods through an inter-canister call
//! rather than as an ingress message.  The two paths differ: `canister_inspect_message` runs
//! only for ingress, so anything a canister must enforce for calls from other canisters has
//! to be tested through a call like this one.
//!
//! Never deploy this canister: it lets any caller make calls with its identity.

use candid::Principal;
use ic_cdk::{call::Call, update};

/// Calls `method` on `canister` with the Candid-encoded `arg`, attaching no cycles.
///
/// Returns the raw Candid-encoded reply, or a description of why the call failed.
#[update]
#[allow(clippy::needless_pass_by_value)] // Update arguments are decoded into owned values.
async fn call_raw(canister: Principal, method: String, arg: Vec<u8>) -> Result<Vec<u8>, String> {
    Call::unbounded_wait(canister, &method)
        .with_raw_args(&arg)
        .await
        .map(ic_cdk::call::Response::into_bytes)
        .map_err(|e| e.to_string())
}
