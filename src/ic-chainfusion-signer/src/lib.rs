use candid::Principal;
use ic_cdk::caller;

#[ic_cdk::update]
#[allow(clippy::needless_pass_by_value)]
fn sign(name: String) -> String {
    let caller = caller();
    let caller_str = if caller == Principal::anonymous() {
        "Anonymous".to_owned()
    } else {
        format!("{caller}")
    };
    format!("Hello, {name} ({caller_str})!")
}

/*
/// Computes a signature for an [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559) transaction.
//#[update(guard = "caller_is_not_anonymous")]
async fn sign_transaction(req: SignRequest) -> String {
    todo!()
}
/// Computes a signature for a hex-encoded message according to [EIP-191](https://eips.ethereum.org/EIPS/eip-191).
//#[update(guard = "caller_is_not_anonymous")]
async fn personal_sign(plaintext: String) -> String {
    todo!()
}
/// Computes a signature for a precomputed hash.
//#[update(guard = "caller_is_not_anonymous")]
async fn sign_prehash(prehash: String) -> String {
    todo!()
}
*/

// Enable Candid export
ic_cdk::export_candid!();
