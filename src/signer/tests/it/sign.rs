use crate::utils::mock::{CALLER, CALLER_ETH_ADDRESS, SEPOLIA_CHAIN_ID};
use crate::utils::pocketic::{setup, PicCanisterTrait};
use candid::{Nat, Principal};
use shared::types::transaction::SignRequest;

#[test]
fn test_sign_transaction() {
    let pic_setup = setup();

    let sign_request: SignRequest = SignRequest {
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

    let transaction = pic_setup.update::<String>(caller, "sign_transaction", sign_request);

    assert_eq!(
        transaction.unwrap(),
        "0x02f86783aa36a7808203158201c87b945e9f1caf942aa8ee887b75f5a6bccaf4b10242480180c080a02fc93932ea116781baffa2f5e62079772c2d6ed91219caff433f653a6e657460a0301f525ac8a55602cc4bddb8c714c2be08aa2bf43fb0ddad974aa4f589d505b9".to_string()
    );
}

#[test]
fn test_personal_sign() {
    let pic_setup = setup();

    let caller = Principal::from_text(CALLER).unwrap();

    let transaction = pic_setup.update::<String>(
        caller,
        "personal_sign",
        hex::encode("test message".to_string()),
    );

    assert_eq!(
        transaction.unwrap(),
        "0xdfa9da1c6e67a7d77b85c8afd0d83cf3b4d095d69055586626be2c83abfdd8423279306df72a7c069b0ad23120765186d0334238a25464d6f705c91e3238418401".to_string()
    );
}

#[test]
fn test_cannot_personal_sign_if_message_is_not_hex_string() {
    let pic_setup = setup();

    let caller = Principal::from_text(CALLER).unwrap();

    let result = pic_setup.update::<String>(caller, "personal_sign", "test message".to_string());

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("failed to decode hex"));
}

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

    let result = pic_setup.update::<String>(caller, "sign_transaction", sign_request);

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .contains("failed to parse the destination address"));
}

#[test]
fn test_anonymous_cannot_sign_transaction() {
    let pic_setup = setup();

    let result = pic_setup.update::<String>(Principal::anonymous(), "sign_transaction", ());

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Anonymous caller not authorized.".to_string()
    );
}

#[test]
fn test_anonymous_cannot_personal_sign() {
    let pic_setup = setup();

    let result = pic_setup.update::<String>(Principal::anonymous(), "personal_sign", ());

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "Anonymous caller not authorized.".to_string()
    );
}
