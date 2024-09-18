use crate::types::{Config, InitArg};
use ic_canister_sig_creation::{extract_raw_root_pk_from_der, IC_ROOT_PK_DER};

impl From<InitArg> for Config {
    /// Creates a new `Config` from the provided `InitArg`.
    ///
    /// # Panics
    /// - If the root key cannot be parsed.
    fn from(arg: InitArg) -> Self {
        let InitArg {
            ecdsa_key_name,
            ic_root_key_der,
        } = arg;
        let ic_root_key_raw = match extract_raw_root_pk_from_der(
            &ic_root_key_der.unwrap_or_else(|| IC_ROOT_PK_DER.to_vec()),
        ) {
            Ok(root_key) => root_key,
            Err(msg) => panic!("{}", format!("Error parsing root key: {msg}")),
        };
        Config {
            ecdsa_key_name,
            ic_root_key_raw: Some(ic_root_key_raw),
        }
    }
}
