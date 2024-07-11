use candid::Principal;
use ic_cdk::caller;

#[ic_cdk::query]
fn greet(name: String) -> String {
    let caller = caller();
    let caller_str = if caller == Principal::anonymous() {
        "Anonymous".to_owned()
    } else {
        format!("{caller}")
    };
    format!("Hello, {} ({})!", name, caller_str)
}
