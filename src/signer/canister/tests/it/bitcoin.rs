use crate::utils::mock::CALLER;
use crate::utils::pocketic::{setup, PicCanisterTrait};
use candid::Principal;
use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;

#[test]
fn test_caller_btc_balance() {
    let pic_setup = setup();

    let caller = Principal::from_text(CALLER).unwrap();
    let network = BitcoinNetwork::Regtest;

    let balance = pic_setup
        .update::<u64>(caller, "caller_btc_balance", network)
        .expect("Failed to call testnet btc balance.");

    assert_eq!(balance, 0);
}
