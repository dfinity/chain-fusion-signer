use crate::{
    canister::{
        cycles_ledger,
        cycles_ledger::ApproveArgs,
        signer::{EthSignPrehashResponse, EthSignTransactionRequest, PaymentType},
    },
    utils::{
        mock::{CALLER, CALLER_ETH_ADDRESS, SEPOLIA_CHAIN_ID},
        pic_canister::PicCanisterTrait,
        pocketic::setup,
        test_environment::TestSetup,
    },
};
use candid::{Nat, Principal};
use ic_chain_fusion_signer_api::types::transaction::SignRequest;

#[test]
fn test_eth_sign_transaction() {
    let test_env = TestSetup::default();

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

    let caller = Principal::from_text(CALLER).unwrap();
    let payment_type = PaymentType::CallerPaysIcrc2Cycles;
    let payment_recipient = cycles_ledger::Account {
        owner: test_env.signer.canister_id(),
        subaccount: None,
    };
    let amount: u64 = 40_000_000_000 + 1000000000;
    test_env
        .ledger
        .icrc_2_approve(caller, &ApproveArgs::new(payment_recipient, amount.into()))
        .expect("Failed to call ledger canister")
        .expect("Failed to approve payment");
    let transaction = test_env
        .signer
        .eth_sign_transaction(caller, sign_request, &Some(payment_type))
        .unwrap();

    assert_eq!(
        transaction.unwrap(),
        EthSignPrehashResponse{ signature: "0x02f86783aa36a7808203158201c87b945e9f1caf942aa8ee887b75f5a6bccaf4b10242480180c001a0fc97df3cb643abb3b565cd95b8d55f108db336612abbb79e0054588587306809a04014551c96d89a90ff89065f08b05772cd582d26e3e1eee4ccf85d2fedc2ad50".to_string()}
    );
}

#[ignore] // TODO: Update this test
#[test]
fn test_personal_sign() {
    let pic_setup = setup();

    let caller = Principal::from_text(CALLER).unwrap();

    let transaction = pic_setup.update_one::<String>(
        caller,
        "personal_sign",
        hex::encode("test message".to_string()),
    );

    assert_eq!(
        transaction.unwrap(),
        "0xdfa9da1c6e67a7d77b85c8afd0d83cf3b4d095d69055586626be2c83abfdd8423279306df72a7c069b0ad23120765186d0334238a25464d6f705c91e3238418401".to_string()
    );
}

#[ignore] // TODO: Update this test
#[test]
fn test_cannot_personal_sign_if_message_is_not_hex_string() {
    let pic_setup = setup();

    let caller = Principal::from_text(CALLER).unwrap();

    let result =
        pic_setup.update_one::<String>(caller, "personal_sign", "test message".to_string());

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
