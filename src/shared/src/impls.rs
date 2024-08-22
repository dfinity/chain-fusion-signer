use crate::types::custom_token::{CustomTokenId, Token};
use crate::types::token::UserToken;
use crate::types::{ApiEnabled, Config, InitArg, MigrationProgress, TokenVersion, Version};
use ic_canister_sig_creation::{extract_raw_root_pk_from_der, IC_ROOT_PK_DER};

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
            api,
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
            api,
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

impl MigrationProgress {
    /// The next phase in the migration process.
    ///
    /// Note: A given phase, such as migrating a `BTreeMap`, may need multiple steps.
    /// The code for that phase will have to keep track of those steps by means of the data in the variant.
    ///
    /// Prior art:
    /// - There is an `enum_iterator` crate, however it deals only with simple enums
    ///   without variant fields.  In this implementation, `next()` always uses the default value for
    ///   the new field, which is always None.  `next()` does NOT step through the values of the
    ///   variant field.
    /// - `strum` has the `EnumIter` derive macro, but that implements `.next()` on an iterator, not on the
    ///   enum itself, so stepping from one variant to the next is not straightforward.
    ///
    /// Note: The next state after Completed is Completed, so the the iterator will run
    /// indefinitely.  In our case returning an option and ending with None would be fine but needs
    /// additional code that we don't need.
    #[must_use]
    pub fn next(&self) -> Self {
        match self {
            MigrationProgress::Pending => MigrationProgress::LockingTarget,
            MigrationProgress::LockingTarget => MigrationProgress::CheckingTarget,
            MigrationProgress::CheckingTarget => MigrationProgress::MigratedUserTokensUpTo(None),
            MigrationProgress::MigratedUserTokensUpTo(_) => {
                MigrationProgress::MigratedCustomTokensUpTo(None)
            }
            MigrationProgress::MigratedCustomTokensUpTo(_) => {
                MigrationProgress::MigratedUserTimestampsUpTo(None)
            }
            MigrationProgress::MigratedUserTimestampsUpTo(_) => {
                MigrationProgress::MigratedUserProfilesUpTo(None)
            }
            MigrationProgress::MigratedUserProfilesUpTo(_) => {
                MigrationProgress::CheckingDataMigration
            }
            MigrationProgress::CheckingDataMigration => MigrationProgress::UnlockingTarget,
            MigrationProgress::UnlockingTarget => MigrationProgress::Unlocking,
            &MigrationProgress::Unlocking | MigrationProgress::Completed => {
                MigrationProgress::Completed
            }
            MigrationProgress::Failed(e) => MigrationProgress::Failed(*e),
        }
    }
}

// `MigrationProgress::next(&self)` should list all the elements in the enum in order, but stop at Completed.
#[test]
fn next_matches_strum_iter() {
    let mut iter = MigrationProgress::iter();
    let mut next = MigrationProgress::Pending;
    while next != MigrationProgress::Completed {
        assert_eq!(iter.next(), Some(next), "iter.next() != Some(next)");
        next = next.next();
    }
    assert_eq!(
        next,
        next.next(),
        "Once completed, it should stay completed"
    );
}
