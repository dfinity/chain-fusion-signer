use crate::utils::{mock::CALLER, pic_canister::PicCanisterTrait, pocketic::setup};
use candid::Principal;
use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;
use ic_chain_fusion_signer_api::types::bitcoin::{
    BitcoinAddressType, GetBalanceError, GetBalanceRequest, GetBalanceResponse,
};

mod caller_balance {
    use super::*;
    #[ignore] // TODO: Update this test
    #[test]
    fn test_caller_btc_balance() {
        let pic_setup = setup();

        let caller = Principal::from_text(CALLER).unwrap();
        let network = BitcoinNetwork::Regtest;
        let params = GetBalanceRequest {
            network,
            address_type: BitcoinAddressType::P2WPKH,
            min_confirmations: None,
        };

        let balance_response = pic_setup
            .update_one::<Result<GetBalanceResponse, GetBalanceError>>(
                caller,
                "btc_caller_balance",
                params,
            )
            .expect("Failed to call testnet btc balance.")
            .expect("Failed to get successul balance response");

        assert_eq!(balance_response.balance, 0u64);
    }
}
