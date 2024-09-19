use candid::Principal;
pub use cycles_ledger_client::Account;
use ic_ledger_types::Subaccount;
use serde_bytes::ByteBuf;

pub mod caller;
pub mod cycles;
pub mod error;
pub mod vendor;
pub use caller::PaymentType;
pub use error::PaymentError;
pub use vendor::Icrc2Payer;

const SUB_ACCOUNT_ZERO: Subaccount = Subaccount([0; 32]);
#[must_use]
pub fn principal2account(principal: &Principal) -> ByteBuf {
    // Note: The AccountIdentifier type contains bytes but has no API to access them.
    // There is a ticket to address this here: https://github.com/dfinity/cdk-rs/issues/519
    // TODO: Simplify this when an API that provides bytes is available.
    let hex_str = ic_ledger_types::AccountIdentifier::new(principal, &SUB_ACCOUNT_ZERO).to_hex();
    hex::decode(&hex_str)
        .unwrap_or_else(|_| {
            unreachable!(
                "Failed to decode hex account identifier we just created: {}",
                hex_str
            )
        })
        .into()
}
