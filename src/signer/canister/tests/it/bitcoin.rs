use candid::Principal;
use ic_chain_fusion_signer_api::methods::SignerMethods;

use crate::{
    canister::{
        cycles_ledger::{self, ApproveArgs},
        signer::{
            BitcoinAddressType, BitcoinNetwork, BtcTxOutput, GetAddressError, GetAddressRequest,
            GetAddressResponse, GetBalanceRequest, GetBalanceResponse, Outpoint, PaymentType,
            SendBtcError, SendBtcRequest, SignBtcResponse, Utxo,
        },
    },
    utils::{
        mock::{
            CALLER_BTC_ADDRESS_MAINNET, CALLER_BTC_ADDRESS_REGTEST, CALLER_BTC_ADDRESS_TESTNET,
        },
        pic_canister::PicCanisterTrait,
        test_environment::{TestSetup, LEDGER_FEE},
    },
};

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
        )
        .expect("Failed to call testnet btc balance.")
        .expect("Failed to get successul balance response");

        assert_eq!(response, GetBalanceResponse { balance: 0u64 });
    }
}

mod address {
    use super::*;

    /// A standard btc_caller_address() call, including payment.
    fn paid_caller_address(
        test_env: &TestSetup,
        caller: Principal,
        request: &GetAddressRequest,
    ) -> Result<Result<GetAddressResponse, GetAddressError>, String> {
        let payment_type = PaymentType::CallerPaysIcrc2Cycles;
        let payment_recipient = cycles_ledger::Account {
            owner: test_env.signer.canister_id(),
            subaccount: None,
        };
        let amount: u64 = SignerMethods::BtcCallerAddress.fee() + LEDGER_FEE as u64;
        test_env
            .ledger
            .icrc_2_approve(caller, &ApproveArgs::new(payment_recipient, amount.into()))
            .expect("Failed to call ledger canister")
            .expect("Failed to approve payment");

        test_env
            .signer
            .btc_caller_address(caller, &request, &Some(payment_type))
    }

    #[test]
    fn test_caller_btc_address_mainnet() {
        let test_env = TestSetup::default();

        let response = paid_caller_address(
            &test_env,
            test_env.user,
            &GetAddressRequest {
                network: BitcoinNetwork::Mainnet,
                address_type: BitcoinAddressType::P2Wpkh,
            },
        )
        .expect("Failed to call testnet btc address.")
        .expect("Failed to get successul btc address response");

        assert_eq!(
            response,
            GetAddressResponse {
                address: CALLER_BTC_ADDRESS_MAINNET.to_string()
            }
        );
    }

    #[test]
    fn test_caller_btc_address_testnet() {
        let test_env = TestSetup::default();

        let response = paid_caller_address(
            &test_env,
            test_env.user,
            &GetAddressRequest {
                network: BitcoinNetwork::Testnet,
                address_type: BitcoinAddressType::P2Wpkh,
            },
        )
        .expect("Failed to call testnet btc address.")
        .expect("Failed to get successul btc address response");

        assert_eq!(
            response,
            GetAddressResponse {
                address: CALLER_BTC_ADDRESS_TESTNET.to_string()
            }
        );
    }

    #[test]
    fn test_caller_btc_address_regtest() {
        let test_env = TestSetup::default();

        let response = paid_caller_address(
            &test_env,
            test_env.user,
            &GetAddressRequest {
                network: BitcoinNetwork::Regtest,
                address_type: BitcoinAddressType::P2Wpkh,
            },
        )
        .expect("Failed to call testnet btc address.")
        .expect("Failed to get successul btc address response");

        assert_eq!(
            response,
            GetAddressResponse {
                address: CALLER_BTC_ADDRESS_REGTEST.to_string()
            }
        );
    }

    #[test]
    fn test_anonymous_cannot_call_btc_address() {
        let test_env = TestSetup::default();

        let response = test_env.signer.btc_caller_address(
            Principal::anonymous(),
            &GetAddressRequest {
                network: BitcoinNetwork::Testnet,
                address_type: BitcoinAddressType::P2Wpkh,
            },
            &Some(PaymentType::CallerPaysIcrc2Cycles),
        );

        assert!(response.is_err());
        assert_eq!(
            response.unwrap_err(),
            "Update call error. RejectionCode: CanisterReject, Error: Update call error. RejectionCode: CanisterReject, Error: Anonymous caller not authorized.".to_string()
        );
    }

    #[test]
    fn test_testnet_btc_address_is_not_same_as_regtest() {
        let test_env = TestSetup::default();

        let testnet_address = paid_caller_address(
            &test_env,
            test_env.user,
            &GetAddressRequest {
                network: BitcoinNetwork::Testnet,
                address_type: BitcoinAddressType::P2Wpkh,
            },
        )
        .expect("Failed to call testnet btc address.")
        .expect("Failed to get successul btc address response")
        .address;

        let regtest_address = paid_caller_address(
            &test_env,
            test_env.user,
            &GetAddressRequest {
                network: BitcoinNetwork::Regtest,
                address_type: BitcoinAddressType::P2Wpkh,
            },
        )
        .expect("Failed to call testnet btc address.")
        .expect("Failed to get successul btc address response")
        .address;

        assert_ne!(testnet_address, regtest_address);
    }
}

mod caller_sign {
    use super::*;

    fn make_test_send_request(network: BitcoinNetwork) -> SendBtcRequest {
        SendBtcRequest {
            network,
            address_type: BitcoinAddressType::P2Wpkh,
            utxos_to_spend: vec![Utxo {
                height: 100,
                value: 100_000,
                outpoint: Outpoint {
                    txid: serde_bytes::ByteBuf::from(vec![
                        0x36, 0xf3, 0xa7, 0xfc, 0xb6, 0xb5, 0xeb, 0xd9, 0xfa, 0x40, 0x41, 0x92,
                        0x8d, 0xa8, 0x9c, 0xd4, 0x23, 0x66, 0x2f, 0x9c, 0x5c, 0x12, 0xe4, 0x1c,
                        0x80, 0xe0, 0x7a, 0x65, 0x59, 0xd1, 0x78, 0xef,
                    ]),
                    vout: 0,
                },
            }],
            fee_satoshis: Some(1000),
            outputs: vec![BtcTxOutput {
                destination_address: CALLER_BTC_ADDRESS_REGTEST.to_string(),
                sent_satoshis: 50_000,
            }],
        }
    }

    /// A standard btc_caller_sign() call, including payment.
    fn paid_caller_sign(
        test_env: &TestSetup,
        caller: Principal,
        request: &SendBtcRequest,
    ) -> Result<Result<SignBtcResponse, SendBtcError>, String> {
        let payment_type = PaymentType::CallerPaysIcrc2Cycles;
        let payment_recipient = cycles_ledger::Account {
            owner: test_env.signer.canister_id(),
            subaccount: None,
        };
        let amount: u64 = SignerMethods::BtcCallerSign.fee() + LEDGER_FEE as u64;
        test_env
            .ledger
            .icrc_2_approve(caller, &ApproveArgs::new(payment_recipient, amount.into()))
            .expect("Failed to call ledger canister")
            .expect("Failed to approve payment");

        test_env
            .signer
            .btc_caller_sign(caller, request, &Some(payment_type))
    }

    #[test]
    fn test_anonymous_cannot_call_btc_sign() {
        let test_env = TestSetup::default();

        let response = test_env.signer.btc_caller_sign(
            Principal::anonymous(),
            &make_test_send_request(BitcoinNetwork::Regtest),
            &Some(PaymentType::CallerPaysIcrc2Cycles),
        );

        assert!(response.is_err());
        assert_eq!(
            response.unwrap_err(),
            "Update call error. RejectionCode: CanisterReject, Error: Update call error. RejectionCode: CanisterReject, Error: Anonymous caller not authorized.".to_string()
        );
    }

    #[test]
    fn test_btc_caller_sign_returns_signed_transaction() {
        let test_env = TestSetup::default();

        let response = paid_caller_sign(
            &test_env,
            test_env.user,
            &make_test_send_request(BitcoinNetwork::Regtest),
        )
        .expect("Failed to call btc_caller_sign")
        .expect("Failed to get successful sign response");

        assert!(!response.signed_transaction_hex.is_empty());
        assert!(!response.txid.is_empty());
        assert!(hex::decode(&response.signed_transaction_hex).is_ok());
    }
}
