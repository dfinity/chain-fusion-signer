use candid::Principal;

/// The schema, which is the first part of a derivation path.
#[repr(u8)]
pub enum Schema {
    /// A simple Eth address, using just the principal and no additional domain separation.
    ///
    /// Please see `from_principal` for details.
    Eth = 1,
}

impl From<Schema> for Vec<u8> {
    fn from(s: Schema) -> Vec<u8> {
        vec![s as u8]
    }
}

/// A derivation path composed of: `vec![schema, principal]`.
pub fn eth(p: &Principal) -> Vec<Vec<u8>> {
    vec![Schema::Eth.into(), p.as_slice().to_vec()]
}
