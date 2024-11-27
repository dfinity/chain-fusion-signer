//! Tests for ethereum methods on the signer canister.
use candid::{Nat, Principal};
use ic_chain_fusion_signer_api::methods::SignerMethods;
use lazy_static::lazy_static;

use crate::{
    canister::{
        cycles_ledger::{self, ApproveArgs},
        signer::{
            EthAddressError, EthAddressRequest, EthAddressResponse, EthPersonalSignRequest,
            EthPersonalSignResponse, EthSignPrehashResponse, EthSignTransactionRequest,
            PaymentType,
        },
    },
    utils::{
        mock::{CALLER_ETH_ADDRESS, SEPOLIA_CHAIN_ID},
        pic_canister::PicCanisterTrait,
        test_environment::{TestSetup, LEDGER_FEE},
    },
};

lazy_static! {
    static ref GOOD_SIGN_TRANSACTION_REQUEST: EthSignTransactionRequest =
        EthSignTransactionRequest {
            chain_id: Nat::from(SEPOLIA_CHAIN_ID),
            to: CALLER_ETH_ADDRESS.to_string(),
            gas: Nat::from(123u64),
            max_fee_per_gas: Nat::from(456u64),
            max_priority_fee_per_gas: Nat::from(789u64),
            value: Nat::from(1u64),
            nonce: Nat::from(0u64),
            data: None,
        };
    static ref GOOD_PERSONAL_SIGN_REQUEST: EthPersonalSignRequest = EthPersonalSignRequest {
        message: hex::encode("test message"),
    };
}

/// Tests for `eth_sign_transaction()`
mod sign_transaction {
    use super::*;
    use crate::canister::signer::EthAddressError;

    /// A standard sign_transaction call, including payment.
    fn paid_sign_transaction(
        test_env: &TestSetup,
        caller: Principal,
        request: &EthSignTransactionRequest,
    ) -> Result<Result<EthSignPrehashResponse, EthAddressError>, String> {
        let payment_type = PaymentType::CallerPaysIcrc2Cycles;
        let payment_recipient = cycles_ledger::Account {
            owner: test_env.signer.canister_id(),
            subaccount: None,
        };
        let amount: u64 = SignerMethods::EthSignTransaction.fee() + LEDGER_FEE as u64;
        test_env
            .ledger
            .icrc_2_approve(caller, &ApproveArgs::new(payment_recipient, amount.into()))
            .expect("Failed to call ledger canister")
            .expect("Failed to approve payment");

        test_env
            .signer
            .eth_sign_transaction(caller, &request, &Some(payment_type))
    }

    #[test]
    fn can_eth_sign_transaction() {
        let test_env = TestSetup::default();
        let response =
            paid_sign_transaction(&test_env, test_env.user, &GOOD_SIGN_TRANSACTION_REQUEST)
                .expect("Failed to call the signer canister")
                .expect("Failed to sign");

        assert_eq!(
        response,
        EthSignPrehashResponse{ signature: "0x02f86783aa36a7808203158201c87b94dfb554b25a5fc2f44aec0fcd8b541f065ac33c0a0180c001a0a31dc737936860aa954f677e540bea59892d1090b07f415b2402e5f3be842b7aa0169a0652a40f997f5c1e3b5f72cfd25d3d56de555452634cd15f3230f727daf6".to_string()}
    );
    }

    #[test]
    fn test_cannot_sign_transaction_with_invalid_to_address() {
        let test_env = TestSetup::default();
        let request = EthSignTransactionRequest {
            to: "invalid_address".to_string(),
            ..GOOD_SIGN_TRANSACTION_REQUEST.clone()
        };
        let response = paid_sign_transaction(&test_env, test_env.user, &request);
        assert!(response.is_err());
        assert!(response
            .unwrap_err()
            .contains("failed to parse the destination address"));
    }

    #[test]
    fn test_anonymous_cannot_sign_transaction() {
        let test_env = TestSetup::default();
        let response = test_env.signer.eth_sign_transaction(
            Principal::anonymous(),
            &GOOD_SIGN_TRANSACTION_REQUEST,
            &Some(PaymentType::CallerPaysIcrc2Cycles),
        );
        assert!(response.is_err());
        assert_eq!(
            response.unwrap_err(),
            "Anonymous caller not authorized.".to_string()
        );
    }
}

/// Tests for `eth_personal_sign()`
mod personal_sign {
    use super::*;

    /// A standard personal_sign call, including payment.
    fn paid_personal_sign(
        test_env: &TestSetup,
        caller: Principal,
        request: &EthPersonalSignRequest,
    ) -> Result<Result<EthPersonalSignResponse, EthAddressError>, String> {
        let payment_type = PaymentType::CallerPaysIcrc2Cycles;
        let payment_recipient = cycles_ledger::Account {
            owner: test_env.signer.canister_id(),
            subaccount: None,
        };
        let amount: u64 = SignerMethods::EthPersonalSign.fee() + LEDGER_FEE as u64;
        test_env
            .ledger
            .icrc_2_approve(caller, &ApproveArgs::new(payment_recipient, amount.into()))
            .expect("Failed to call ledger canister")
            .expect("Failed to approve payment");

        test_env
            .signer
            .eth_personal_sign(caller, &request, &Some(payment_type))
    }

    #[test]
    fn can_eth_personal_sign() {
        let test_env = TestSetup::default();
        let response = paid_personal_sign(&test_env, test_env.user, &GOOD_PERSONAL_SIGN_REQUEST)
            .expect("Failed to reach signer canister")
            .expect("Failed to sign");

        assert_eq!(
            response,
            EthPersonalSignResponse {
                signature: "0xd7a6d8259a2c2c64838c425751d9555c7ab4c12bafc266e5a436ddff69f8deff49ef842852b4154c0d32aa7d719ce0e87bba70904121b0ef214f5faf1963c62400".to_string()
            }
        );
    }

    #[test]
    fn cannot_personal_sign_if_message_is_not_hex_string() {
        let test_env = TestSetup::default();
        let request = EthPersonalSignRequest {
            message: "test message".to_string(), /* Note: This should be a hex string.  Let'
                                                  * stest what happens when it's not. */
        };
        let response = paid_personal_sign(&test_env, test_env.user, &request);
        assert!(response.is_err());
        assert!(response.unwrap_err().contains("failed to decode hex"));
    }

    #[test]
    fn test_anonymous_cannot_personal_sign() {
        let test_env = TestSetup::default();

        let caller = Principal::anonymous();
        let payment_type = PaymentType::CallerPaysIcrc2Cycles;

        let result = test_env.signer.eth_personal_sign(
            caller,
            &GOOD_PERSONAL_SIGN_REQUEST,
            &Some(payment_type),
        );

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Anonymous caller not authorized.".to_string()
        );
    }
}

/// Tests for `eth_address()`
mod eth_address {
    use super::*;

    /// A standard eth_address call, including payment.
    fn paid_eth_address(
        test_env: &TestSetup,
        caller: Principal,
        principal: Option<Principal>,
    ) -> Result<Result<EthAddressResponse, EthAddressError>, String> {
        let payment_type = PaymentType::CallerPaysIcrc2Cycles;
        let payment_recipient = cycles_ledger::Account {
            owner: test_env.signer.canister_id(),
            subaccount: None,
        };
        let amount: u64 = SignerMethods::EthAddress.fee() + LEDGER_FEE as u64;
        test_env
            .ledger
            .icrc_2_approve(caller, &ApproveArgs::new(payment_recipient, amount.into()))
            .expect("Failed to call ledger canister")
            .expect("Failed to approve payment");

        test_env.signer.eth_address(
            caller,
            &EthAddressRequest { principal },
            &Some(payment_type),
        )
    }

    #[test]
    fn test_caller_eth_address() {
        let test_env = TestSetup::default();
        let response = paid_eth_address(&test_env, test_env.user, None)
            .expect("Failed to reach signer canister")
            .expect("Failed to get eth address.");

        assert_eq!(
            response,
            EthAddressResponse {
                address: "0x9f826268a4a9F25033b777ADE2F377244c5ec530".to_string()
            }
        );
    }

    #[test]
    fn test_eth_address_of() {
        let test_env = TestSetup::default();
        let response = paid_eth_address(&test_env, test_env.user, Some(test_env.user))
            .expect("Failed to reach signer canister")
            .expect("Failed to get eth address.");

        assert_eq!(
            response,
            EthAddressResponse {
                address: CALLER_ETH_ADDRESS.to_string()
            }
        );
    }

    #[test]
    fn test_anonymous_cannot_call_eth_address() {
        let test_env = TestSetup::default();
        let response = test_env.signer.eth_address(
            Principal::anonymous(),
            &EthAddressRequest {
                principal: Some(test_env.user),
            },
            &Some(PaymentType::CallerPaysIcrc2Cycles),
        );
        assert!(response.is_err());
        assert_eq!(
            response.unwrap_err(),
            "Anonymous caller not authorized.".to_string()
        );
    }

    #[test]
    fn test_cannot_call_eth_address_of_for_anonymous() {
        let test_env = TestSetup::default();
        let response = test_env.signer.eth_address(
            test_env.user,
            &EthAddressRequest {
                principal: Some(Principal::anonymous()),
            },
            &Some(PaymentType::CallerPaysIcrc2Cycles),
        );
        assert!(response.is_err());
        assert!(response
            .unwrap_err()
            .contains("Anonymous principal is not authorized"));
    }
}
