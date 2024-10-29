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
    pub fn fee(&self) -> u64 {
        // Note: Fees are determined with the aid of scripts/check-pricing
        match self {
            SignerMethods::GenericCallerEcdsaPublicKey => 1_000_000_000,
            SignerMethods::GenericSignWithEcdsa => 40_000_000_000,
            SignerMethods::EthAddress => 1_000_000_000,
            SignerMethods::EthAddressOfCaller => 1_000_000_000,
            SignerMethods::EthSignTransaction => 40_000_000_000,
            SignerMethods::EthPersonalSign => 40_000_000_000,
            SignerMethods::EthSignPrehash => 40_000_000_000,
            SignerMethods::BtcCallerAddress => 20_000_000,
            SignerMethods::BtcCallerBalance => 40_000_000,
            SignerMethods::BtcCallerSend => 130_000_000_000,
        }
    }
}
