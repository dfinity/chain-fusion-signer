use crate::types::custom_token::{CustomTokenId, Token};
use crate::types::token::UserToken;
use crate::types::{ApiEnabled, Config, InitArg, TokenVersion, Version};
use ic_canister_sig_creation::{extract_raw_root_pk_from_der, IC_ROOT_PK_DER};
#[cfg(test)]
use strum::IntoEnumIterator;

impl From<&Token> for CustomTokenId {
    fn from(token: &Token) -> Self {
        match token {
            Token::Icrc(token) => CustomTokenId::Icrc(token.ledger_id),
        }
    }
}

impl TokenVersion for UserToken {
    fn get_version(&self) -> Option<Version> {
        self.version
    }

    fn clone_with_incremented_version(&self) -> Self {
        let mut cloned = self.clone();
        cloned.version = Some(cloned.version.unwrap_or_default() + 1);
        cloned
    }

    fn clone_with_initial_version(&self) -> Self {
        let mut cloned = self.clone();
        cloned.version = Some(1);
        cloned
    }
}

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

impl Default for ApiEnabled {
    fn default() -> Self {
        Self::Enabled
    }
}
impl ApiEnabled {
    #[must_use]
    pub fn readable(&self) -> bool {
        matches!(self, Self::Enabled | Self::ReadOnly)
    }
    #[must_use]
    pub fn writable(&self) -> bool {
        matches!(self, Self::Enabled)
    }
}
#[test]
fn test_api_enabled() {
    assert_eq!(ApiEnabled::Enabled.readable(), true);
    assert_eq!(ApiEnabled::Enabled.writable(), true);
    assert_eq!(ApiEnabled::ReadOnly.readable(), true);
    assert_eq!(ApiEnabled::ReadOnly.writable(), false);
    assert_eq!(ApiEnabled::Disabled.readable(), false);
    assert_eq!(ApiEnabled::Disabled.writable(), false);
}
