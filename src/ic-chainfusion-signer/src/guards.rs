use crate::read_config;
use candid::Principal;
use ic_cdk::api::is_controller;
use ic_cdk::caller;

pub fn caller_is_not_anonymous() -> Result<(), String> {
    if caller() == Principal::anonymous() {
        Err("Anonymous caller not authorized.".to_string())
    } else {
        Ok(())
    }
}

pub fn caller_is_allowed() -> Result<(), String> {
    let caller = caller();
    if is_controller(&caller) {
        Ok(())
    } else {
        Err("Caller is not allowed.".to_string())
    }
}

/// Is getting threshold public keys is enabled?
pub fn may_read_threshold_keys() -> Result<(), String> {
    caller_is_not_anonymous()?;
    if read_config(|s| s.api.unwrap_or_default().threshold_key.readable()) {
        Ok(())
    } else {
        Err("Reading threshold keys is disabled.".to_string())
    }
}
/// Caller is allowed AND reading threshold keys is enabled.
pub fn caller_is_allowed_and_may_read_threshold_keys() -> Result<(), String> {
    caller_is_allowed()?;
    may_read_threshold_keys()
}

/// Is signing with threshold keys is enabled?
pub fn may_threshold_sign() -> Result<(), String> {
    caller_is_not_anonymous()?;
    if read_config(|s| s.api.unwrap_or_default().threshold_key.writable()) {
        Ok(())
    } else {
        Err("Threshold signing is disabled.".to_string())
    }
}
