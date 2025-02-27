use ic_canister_sig_creation::{extract_raw_root_pk_from_der, IC_ROOT_PK_DER};
use ic_papi_api::PaymentError;

use crate::types::{
    bitcoin::{GetAddressError, GetBalanceError, SendBtcError},
    Config, InitArg,
};

impl From<InitArg> for Config {
    /// Creates a new `Config` from the provided `InitArg`.
    ///
    /// # Panics
    /// - If the root key cannot be parsed.
    fn from(arg: InitArg) -> Self {
        let InitArg {
            ecdsa_key_name,
            ic_root_key_der,
            cycles_ledger,
        } = arg;
        let ic_root_key_raw = match extract_raw_root_pk_from_der(
            &ic_root_key_der.unwrap_or_else(|| IC_ROOT_PK_DER.to_vec()),
        ) {
            Ok(root_key) => root_key,
            Err(msg) => panic!("{}", format!("Error parsing root key: {msg}")),
        };
        let cycles_ledger =
            cycles_ledger.unwrap_or_else(ic_papi_api::cycles::cycles_ledger_canister_id);
        Config {
            ecdsa_key_name,
            ic_root_key_raw: Some(ic_root_key_raw),
            cycles_ledger,
        }
    }
}

impl From<PaymentError> for GetAddressError {
    fn from(e: PaymentError) -> Self {
        GetAddressError::PaymentError(e)
    }
}

impl From<PaymentError> for GetBalanceError {
    fn from(e: PaymentError) -> Self {
        GetBalanceError::PaymentError(e)
    }
}

impl From<PaymentError> for SendBtcError {
    fn from(e: PaymentError) -> Self {
        SendBtcError::PaymentError(e)
    }
}
