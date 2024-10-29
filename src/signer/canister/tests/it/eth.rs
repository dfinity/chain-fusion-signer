use crate::{
    canister::{
        cycles_ledger::{self, ApproveArgs},
        signer::{
            EthPersonalSignRequest, EthPersonalSignResponse, EthSignPrehashResponse,
            EthSignTransactionRequest, PaymentType,
        },
    },
    utils::{
        mock::{CALLER, CALLER_ETH_ADDRESS, SEPOLIA_CHAIN_ID},
        pic_canister::PicCanisterTrait,
        pocketic::setup,
        test_environment::{TestSetup, LEDGER_FEE},
    },
};
use candid::{Nat, Principal};
use ic_chain_fusion_signer_api::{methods::SignerMethods, types::transaction::SignRequest};

#[test]
fn can_eth_sign_transaction() {
    let test_env = TestSetup::default();

    let caller = Principal::from_text(CALLER).unwrap();

    let sign_request = &EthSignTransactionRequest {
        chain_id: Nat::from(SEPOLIA_CHAIN_ID),
        to: CALLER_ETH_ADDRESS.to_string(),
        gas: Nat::from(123u64),
        max_fee_per_gas: Nat::from(456u64),
        max_priority_fee_per_gas: Nat::from(789u64),
        value: Nat::from(1u64),
        nonce: Nat::from(0u64),
        data: None,
    };

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

    let response = test_env
        .signer
        .eth_sign_transaction(caller, sign_request, &Some(payment_type))
        .expect("Failed to reach the signer canister")
        .expect("Failed to sign");

    assert_eq!(
        response,
        EthSignPrehashResponse{ signature: "0x02f86783aa36a7808203158201c87b945e9f1caf942aa8ee887b75f5a6bccaf4b10242480180c001a0fc97df3cb643abb3b565cd95b8d55f108db336612abbb79e0054588587306809a04014551c96d89a90ff89065f08b05772cd582d26e3e1eee4ccf85d2fedc2ad50".to_string()}
    );
}

#[test]
fn can_eth_personal_sign() {
    let test_env = TestSetup::default();

    let caller = Principal::from_text(CALLER).unwrap();

    let request = EthPersonalSignRequest {
        message: hex::encode("test message"),
    };

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

    let result = test_env
        .signer
        .eth_personal_sign(caller, &request, &Some(payment_type))
        .expect("Failed to call the signer canister")
        .expect("Failed to sign");

    assert_eq!(
        result,
        EthPersonalSignResponse {
            signature: "0x91f0caeca09d8520c905be5287e3fd13fcd355f17fdec41d72430b5bd6c5274266a2840f693377c853f36bc7b82f9e353d8da53e2c8530250f85adf5551268e800".to_string()
        }
    );
}

#[test]
fn cannot_personal_sign_if_message_is_not_hex_string() {
    let test_env = TestSetup::default();

    let caller = Principal::from_text(CALLER).unwrap();

    let request = EthPersonalSignRequest {
        message: "test message".to_string(),
    };

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

    let result = test_env
        .signer
        .eth_personal_sign(caller, &request, &Some(payment_type));

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("failed to decode hex"));
}

#[ignore] // TODO: Update this test
#[test]
fn test_cannot_sign_transaction_with_invalid_to_address() {
    let pic_setup = setup();

    let sign_request: SignRequest = SignRequest {
        chain_id: Nat::from(SEPOLIA_CHAIN_ID),
        to: "invalid_address".to_string(),
        gas: Nat::from(123u64),
        max_fee_per_gas: Nat::from(456u64),
        max_priority_fee_per_gas: Nat::from(789u64),
        value: Nat::from(1u64),
        nonce: Nat::from(0u64),
        data: None,
    };

    let caller = Principal::from_text(CALLER).unwrap();

    let result = pic_setup.update_one::<String>(caller, "sign_transaction", sign_request);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("failed to parse the destination address"));
}

#[ignore] // TODO: Update this test
#[test]
fn test_anonymous_cannot_sign_transaction() {
    let pic_setup = setup();

    let result = pic_setup.update_one::<String>(Principal::anonymous(), "sign_transaction", ());

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Anonymous caller not authorized.".to_string()
    );
}

#[ignore] // TODO: Update this test
#[test]
fn test_anonymous_cannot_personal_sign() {
    let pic_setup = setup();

    let result = pic_setup.update_one::<String>(Principal::anonymous(), "personal_sign", ());

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Anonymous caller not authorized.".to_string()
    );
}
