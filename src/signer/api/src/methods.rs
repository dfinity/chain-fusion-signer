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
}

impl SignerMethods {
    /// The cost, in cycles, of every paid chain fusion signer API method.
    #[must_use]
    pub fn fee(&self) -> u64 {
        // Note: Fees are determined with the aid of scripts/check-pricing
        match self {
            SignerMethods::GenericCallerEcdsaPublicKey
            | SignerMethods::EthAddress
            | SignerMethods::EthAddressOfCaller => 1_000_000_000,
            SignerMethods::GenericSignWithEcdsa
            | SignerMethods::EthSignTransaction
            | SignerMethods::EthPersonalSign
            | SignerMethods::EthSignPrehash => 40_000_000_000,
            SignerMethods::BtcCallerAddress => 20_000_000,
            SignerMethods::BtcCallerBalance => 40_000_000,
            SignerMethods::BtcCallerSend => 130_000_000_000,
        }
    }
}
