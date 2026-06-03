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
    BtcCallerSign,
    SchnorrPublicKey,
    SchnorrSign,
}

impl SignerMethods {
    /// The fixed-component cost, in cycles, of every paid chain fusion signer API method.
    ///
    /// For `BtcCallerSign` and `BtcCallerSend`, the underlying work scales with the
    /// number of transaction inputs (one `sign_with_ecdsa` per input). The value
    /// returned here is the per-call overhead only; the actual fee charged is
    /// `fee() + n_inputs * per_input_fee()`. Use [`Self::fee_for_inputs`] to compute
    /// the total in one call.
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn fee(&self) -> u128 {
        // Note: Fees are determined with the aid of scripts/check-pricing
        match self {
            SignerMethods::BtcCallerAddress => 79_000_000,
            SignerMethods::BtcCallerBalance => 113_000_000,
            SignerMethods::BtcCallerSend => 95_000_000_000,
            SignerMethods::BtcCallerSign => 74_000_000_000,
            SignerMethods::EthAddress | SignerMethods::EthAddressOfCaller => 77_000_000,
            SignerMethods::EthPersonalSign => 37_000_000_000,
            SignerMethods::EthSignPrehash => 37_000_000_000,
            SignerMethods::EthSignTransaction => 37_000_000_000,
            SignerMethods::GenericCallerEcdsaPublicKey => 77_000_000,
            SignerMethods::GenericSignWithEcdsa => 37_000_000_000,
            SignerMethods::SchnorrPublicKey => 77_000_000,
            SignerMethods::SchnorrSign => 37_000_000_000,
        }
    }

    /// The cost, in cycles, charged per transaction input for methods whose work
    /// scales linearly with the number of inputs.
    ///
    /// `BtcCallerSign` and `BtcCallerSend` each run one `sign_with_ecdsa` call per
    /// input; the value here covers that signature plus margin. All other methods
    /// return `0`.
    #[must_use]
    pub fn per_input_fee(&self) -> u128 {
        match self {
            SignerMethods::BtcCallerSign | SignerMethods::BtcCallerSend => 37_000_000_000,
            _ => 0,
        }
    }

    /// Total fee for a call that processes `n_inputs` transaction inputs.
    ///
    /// For methods that do not scale with input count, the result equals
    /// [`Self::fee`] regardless of `n_inputs`.
    #[must_use]
    pub fn fee_for_inputs(&self, n_inputs: u64) -> u128 {
        self.fee() + u128::from(n_inputs) * self.per_input_fee()
    }
}
