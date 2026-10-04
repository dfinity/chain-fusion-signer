use candid::Principal;
use ic_cdk::api::{is_controller, msg_caller};

pub fn caller_is_not_anonymous() -> Result<(), String> {
    if msg_caller() == Principal::anonymous() {
        Err("Update call error. RejectionCode: CanisterReject, Error: Anonymous caller not authorized.".to_string())
    } else {
        Ok(())
    }
}

pub fn caller_is_controller() -> Result<(), String> {
    if is_controller(&msg_caller()) {
        Ok(())
    } else {
        Err(
            "Update call error. RejectionCode: CanisterReject, Error: Caller is not a controller."
                .to_string(),
        )
    }
}
