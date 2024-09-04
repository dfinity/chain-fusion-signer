use candid::Principal;

/// The schema, which is the first part of a derivation path.
#[repr(u8)]
pub enum Schema {
    /// A principal's default Btc address.
    ///
    /// Please see `from_principal` for details.
    Btc = 0,
    /// A principal's default Eth address.
    ///
    /// Please see `from_principal` for details.
    Eth = 1,
}

impl From<Schema> for Vec<u8> {
    fn from(s: Schema) -> Vec<u8> {
        vec![s as u8]
    }
}

/// The default derivation path for a principal signing Ethereum transactions.
pub fn eth(p: &Principal) -> Vec<Vec<u8>> {
    vec![Schema::Eth.into(), p.as_slice().to_vec()]
}

/// The default derivation path for a principal signing Bitcoin transactions.
pub fn btc(p: &Principal) -> Vec<Vec<u8>> {
    vec![Schema::Btc.into(), p.as_slice().to_vec()]
}
