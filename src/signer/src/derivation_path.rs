use candid::Principal;

/// The schema, which is the first part of a derivation path.
#[repr(u8)]
pub enum Schema {
    /// The derivation path for a principal, with no further domain separation.
    ///
    /// Please see `from_principal` for details.
    Principal = 1,
}

impl From<Schema> for Vec<u8> {
    fn from(s: Schema) -> Vec<u8> {
        vec![s as u8]
    }
}

/// A derivation path composed of: `vec![schema, principal]`.
pub fn from_principal(p: &Principal) -> Vec<Vec<u8>> {
    vec![Schema::Principal.into(), p.as_slice().to_vec()]
}
