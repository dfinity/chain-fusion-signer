use crate::utils::{
    mock::{
        CALLER, CALLER_BTC_ADDRESS_MAINNET, CALLER_BTC_ADDRESS_REGTEST, CALLER_BTC_ADDRESS_TESTNET,
        CALLER_ETH_ADDRESS,
    },
    pic_canister::PicCanisterTrait,
    pocketic::setup,
};
use candid::Principal;
use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;
use ic_chain_fusion_signer_api::types::bitcoin::{
    BitcoinAddressType, GetAddressError, GetAddressRequest, GetAddressResponse,
};

#[ignore] // TODO: Update this test
#[test]
fn test_eth_address_of() {
    let pic_setup = setup();

    let caller = Principal::from_text(CALLER).unwrap();

    let address = pic_setup
        .update_one::<String>(caller, "eth_address_of", caller)
        .expect("Failed to call eth address of.");

    assert_eq!(address, CALLER_ETH_ADDRESS.to_string());
}

#[ignore] // TODO: Update this test
#[test]
fn test_anonymous_cannot_call_eth_address() {
    let pic_setup = setup();

    let address = pic_setup.update_one::<String>(Principal::anonymous(), "caller_eth_address", ());

    assert!(address.is_err());
    assert_eq!(
        address.unwrap_err(),
        "Anonymous caller not authorized.".to_string()
    );
}

#[ignore] // TODO: Update this test
#[test]
fn test_cannot_call_eth_address_of_for_anonymous() {
    let pic_setup = setup();

    let caller = Principal::from_text(CALLER).unwrap();

    let address = pic_setup.update_one::<String>(caller, "eth_address_of", Principal::anonymous());

    assert!(address.is_err());
    assert!(address
        .unwrap_err()
        .contains("Anonymous principal is not authorized"));
}

#[ignore] // TODO: Update this test
#[test]
fn test_caller_btc_address_mainnet() {
    let pic_setup = setup();

    let caller = Principal::from_text(CALLER).unwrap();
    let network = BitcoinNetwork::Mainnet;
    let params = GetAddressRequest {
        network,
        address_type: BitcoinAddressType::P2WPKH,
    };

    let address_response = pic_setup
        .update_one::<Result<GetAddressResponse, GetAddressError>>(
            caller,
            "btc_caller_address",
            params,
        )
        .expect("Failed to call testnet btc address.")
        .expect("Failed to get successful response");

    assert_eq!(
        address_response.address,
        CALLER_BTC_ADDRESS_MAINNET.to_string()
    );
}

#[ignore] // TODO: Update this test
#[test]
fn test_caller_btc_address_testnet() {
    let pic_setup = setup();

    let caller = Principal::from_text(CALLER).unwrap();
    let network = BitcoinNetwork::Testnet;
    let params = GetAddressRequest {
        network,
        address_type: BitcoinAddressType::P2WPKH,
    };

    let address_response = pic_setup
        .update_one::<Result<GetAddressResponse, GetAddressError>>(
            caller,
            "btc_caller_address",
            params,
        )
        .expect("Failed to call testnet btc address.")
        .expect("Failed to get successful response");

    assert_eq!(
        address_response.address,
        CALLER_BTC_ADDRESS_TESTNET.to_string()
    );
}

#[ignore] // TODO: Update this test
#[test]
fn test_caller_btc_address_regtest() {
    let pic_setup = setup();

    let caller = Principal::from_text(CALLER).unwrap();
    let network = BitcoinNetwork::Regtest;
    let params = GetAddressRequest {
        network,
        address_type: BitcoinAddressType::P2WPKH,
    };

    let address_response = pic_setup
        .update_one::<Result<GetAddressResponse, GetAddressError>>(
            caller,
            "btc_caller_address",
            params,
        )
        .expect("Failed to call testnet btc address.")
        .expect("Failed to get successful response");

    assert_eq!(
        address_response.address,
        CALLER_BTC_ADDRESS_REGTEST.to_string()
    );
}

#[ignore] // TODO: Update this test
#[test]
fn test_anonymous_cannot_call_btc_address() {
    let pic_setup = setup();
    let network = BitcoinNetwork::Testnet;
    let params = GetAddressRequest {
        network,
        address_type: BitcoinAddressType::P2WPKH,
    };

    let address =
        pic_setup.update_one::<String>(Principal::anonymous(), "btc_caller_address", params);

    assert!(address.is_err());
    assert_eq!(
        address.unwrap_err(),
        "Anonymous caller not authorized.".to_string()
    );
}

#[ignore] // TODO: Update this test
#[test]
fn test_testnet_btc_address_is_not_same_as_regtest() {
    let pic_setup = setup();

    let caller = Principal::from_text(CALLER).unwrap();
    let params_testnet = GetAddressRequest {
        network: BitcoinNetwork::Testnet,
        address_type: BitcoinAddressType::P2WPKH,
    };
    let params_regtest = GetAddressRequest {
        network: BitcoinNetwork::Regtest,
        address_type: BitcoinAddressType::P2WPKH,
    };

    let address_response_testnet = pic_setup
        .update_one::<Result<GetAddressResponse, GetAddressError>>(
            caller,
            "btc_caller_address",
            params_testnet,
        )
        .expect("Failed to call testnet btc address.")
        .expect("Failed to get successful response");

    let address_response_regtest = pic_setup
        .update_one::<Result<GetAddressResponse, GetAddressError>>(
            caller,
            "btc_caller_address",
            params_regtest,
        )
        .expect("Failed to call testnet btc address.")
        .expect("Failed to get successful response");

    assert_ne!(
        address_response_testnet.address,
        address_response_regtest.address
    );
}
