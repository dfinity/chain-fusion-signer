use candid::Principal;

/// The schema, which is the first part of a derivation path.
#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    /// The first part of a derivation path is the schema.  Typically a single byte but may be any number of bytes.
    fn from(s: Schema) -> Vec<u8> {
        vec![s as u8]
    }
}

impl Schema {
    /// The caller may specify any derivation path, as long as it starts with schema and caller principal.
    pub fn derivation_path(&self, principal: &Principal) -> Vec<Vec<u8>> {
        vec![(*self).into(), principal.as_slice().to_vec()]
    }
}
