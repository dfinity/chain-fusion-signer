use crate::utils::{mock::CALLER, pocketic::setup};
use candid::Principal;
use crate::utils::{
    mock::{
        CALLER_BTC_ADDRESS_MAINNET, CALLER_BTC_ADDRESS_REGTEST, CALLER_BTC_ADDRESS_TESTNET,
    },
};
use crate::{
    canister::{
        cycles_ledger::{self, ApproveArgs},
        signer::{
            EthAddressError, EthAddressRequest, EthAddressResponse, EthPersonalSignRequest,
            EthPersonalSignResponse, EthSignPrehashResponse, EthSignTransactionRequest,
            PaymentType, GetBalanceRequest, GetBalanceResponse, GetAddressError, GetAddressRequest, GetAddressResponse, BitcoinAddressType, BitcoinNetwork,
        },
    },
    utils::{
        mock::{CALLER_ETH_ADDRESS, SEPOLIA_CHAIN_ID},
        pic_canister::PicCanisterTrait,
        test_environment::{TestSetup, LEDGER_FEE},
    },
};
use ic_chain_fusion_signer_api::methods::SignerMethods;


mod caller_balance {
    use super::*;

        /// A standard btc_caller_balance() call, including payment.
        fn paid_caller_balance(
            test_env: &TestSetup,
            caller: Principal,
            request: &GetBalanceRequest,
        ) -> Result<Result<GetBalanceResponse, GetAddressError>, String> {
            let payment_type = PaymentType::CallerPaysIcrc2Cycles;
            let payment_recipient = cycles_ledger::Account {
                owner: test_env.signer.canister_id(),
                subaccount: None,
            };
            let amount: u64 = SignerMethods::BtcCallerBalance.fee() + LEDGER_FEE as u64;
            test_env
                .ledger
                .icrc_2_approve(caller, &ApproveArgs::new(payment_recipient, amount.into()))
                .expect("Failed to call ledger canister")
                .expect("Failed to approve payment");
    
            test_env
                .signer
                .btc_caller_balance(caller, &request, &Some(payment_type))
        }

    #[test]
    fn test_caller_btc_balance() {
        let test_env = TestSetup::default();

        let response = paid_caller_balance(
            &test_env,
            test_env.user,
            &GetBalanceRequest {
                network: BitcoinNetwork::Regtest,
                address_type: BitcoinAddressType::P2Wpkh,
                min_confirmations: None,
            },
        )            .expect("Failed to call testnet btc balance.")
            .expect("Failed to get successul balance response");

        assert_eq!(response, GetBalanceResponse{balance: 0u64});
    }
}

mod address {
    use super::*;
    
    #[ignore] // TODO: Update this test
    #[test]
    fn test_caller_btc_address_mainnet() {
        let pic_setup = setup();
    
        let caller = Principal::from_text(CALLER).unwrap();
        let network = BitcoinNetwork::Mainnet;
        let params = GetAddressRequest {
            network,
            address_type: BitcoinAddressType::P2Wpkh,
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
            address_type: BitcoinAddressType::P2Wpkh,
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
            address_type: BitcoinAddressType::P2Wpkh,
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
            address_type: BitcoinAddressType::P2Wpkh,
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
            address_type: BitcoinAddressType::P2Wpkh,
        };
        let params_regtest = GetAddressRequest {
            network: BitcoinNetwork::Regtest,
            address_type: BitcoinAddressType::P2Wpkh,
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
}