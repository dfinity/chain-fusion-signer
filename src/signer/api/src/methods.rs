pub enum SignerMethods {
    GenericCallerEcdsaPublicKey,
    GenericSignWithEcdsa,
    EthAddress,
    EthAddressOfCaller,
    EthSignTransaction,
    EthPersonalSign,
    EthSignPrehash,
    BtcCallerAddress,
    BtcCallerBalance,
    BtcCallerSend,
    SchnorrPublicKey,
    SchnorrSign,
}

impl SignerMethods {
    /// The cost, in cycles, of every paid chain fusion signer API method.
    #[must_use]
    #[allow(clippy::match_same_arms)] // Entries are sorted by method, as this makes them easier to manage.
    pub fn fee(&self) -> u64 {
        // Note: Fees are determined with the aid of scripts/check-pricing
        match self {
            SignerMethods::BtcCallerAddress => 79000000,
            SignerMethods::BtcCallerBalance => 113000000,
            SignerMethods::BtcCallerSend => 132000000000,
            SignerMethods::EthAddress | SignerMethods::EthAddressOfCaller => 77000000,
            SignerMethods::EthPersonalSign => 37000000000,
            SignerMethods::EthSignPrehash => 37000000000,
            SignerMethods::EthSignTransaction => 37000000000,
            SignerMethods::GenericCallerEcdsaPublicKey => 77000000,
            SignerMethods::GenericSignWithEcdsa => 37000000000,
            SignerMethods::SchnorrPublicKey => 77000000,
            SignerMethods::SchnorrSign => 37000000000,
        }
    }
}
