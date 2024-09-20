use crate::utils::mock::CALLER;
use crate::utils::pocketic::{setup, PicCanisterTrait};
use candid::{Nat, Principal};
use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;
use ic_chain_fusion_signer_api::types::bitcoin::{
    BitcoinAddressType, GetBalanceError, GetBalanceRequest, GetBalanceResponse,
};

#[test]
fn test_caller_btc_balance() {
    let pic_setup = setup();

    let caller = Principal::from_text(CALLER).unwrap();
    let network = BitcoinNetwork::Regtest;
    let params = GetBalanceRequest {
        network,
        address_type: BitcoinAddressType::P2WPKH,
    };

    let balance_response = pic_setup
        .update::<Result<GetBalanceResponse, GetBalanceError>>(caller, "caller_btc_balance", params)
        .expect("Failed to call testnet btc balance.")
        .expect("Failed to get successul balance response");

    assert_eq!(balance_response.balance, Nat::from(0u128));
}
