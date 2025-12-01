use candid::Principal;
use ic_cdk::api::msg_caller;

#[ic_cdk::query]
#[allow(clippy::needless_pass_by_value)]
fn greet(name: String) -> String {
    let caller = msg_caller();
    let caller_str = if caller == Principal::anonymous() {
        "Anonymous".to_owned()
    } else {
        format!("{caller}")
    };
    format!("Hello, {name} ({caller_str})!")
}
