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
    /// The cost, in cycles, of every paid chain fusion signer API method.
    ///
    /// For most methods this is the exact fee deducted by the canister.
    ///
    /// For `BtcCallerSign` and `BtcCallerSend` the deducted fee depends on the number
    /// of transaction inputs (one `sign_with_ecdsa` per input). The value returned here
    /// is a **grace-period default** sized for a 2-input transaction; it is intended
    /// to give existing callers (who pre-approve a single cycle amount) a soft landing
    /// while they migrate. The canister itself deducts the precise amount
    /// `btc_base_fee() + n_inputs * btc_per_input_fee()`. Callers that handle BTC sign
    /// or send should compute the total with [`Self::btc_fee_for_inputs`] rather than
    /// relying on `fee()`.
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn fee(&self) -> u128 {
        // Note: Fees are determined with the aid of scripts/check-pricing
        match self {
            SignerMethods::BtcCallerAddress => 79_000_000,
            SignerMethods::BtcCallerBalance => 113_000_000,
            // Grace-period default sized for a 2-input transaction:
            // btc_base_fee() + 2 * btc_per_input_fee() = 95 B + 2 * 37 B = 169 B
            SignerMethods::BtcCallerSend => 169_000_000_000,
            // Grace-period default sized for a 2-input transaction:
            // btc_base_fee() + 2 * btc_per_input_fee() = 74 B + 2 * 37 B = 148 B
            SignerMethods::BtcCallerSign => 148_000_000_000,
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

    /// The per-call base fee, in cycles, for BTC sign/send methods.
    ///
    /// Returns the fixed per-call overhead for `BtcCallerSign` (74 B) and
    /// `BtcCallerSend` (95 B). For all other methods the base fee equals
    /// [`Self::fee`].
    #[must_use]
    pub fn btc_base_fee(&self) -> u128 {
        match self {
            SignerMethods::BtcCallerSign => 74_000_000_000,
            SignerMethods::BtcCallerSend => 95_000_000_000,
            _ => self.fee(),
        }
    }

    /// The cost, in cycles, charged per BTC transaction input.
    ///
    /// `BtcCallerSign` and `BtcCallerSend` each run one `sign_with_ecdsa` call per
    /// input in `utxos_to_spend`; the value here covers that signature plus margin.
    /// All other methods return `0`.
    #[must_use]
    pub fn btc_per_input_fee(&self) -> u128 {
        match self {
            SignerMethods::BtcCallerSign | SignerMethods::BtcCallerSend => 37_000_000_000,
            _ => 0,
        }
    }

    /// Total fee for a BTC sign/send call that processes `n_inputs` transaction inputs.
    ///
    /// For non-BTC methods this returns [`Self::fee`] regardless of `n_inputs`, so it
    /// is safe to call on any variant, but the name reflects its intended use with
    /// `BtcCallerSign` and `BtcCallerSend`.
    #[must_use]
    pub fn btc_fee_for_inputs(&self, n_inputs: u64) -> u128 {
        self.btc_base_fee() + u128::from(n_inputs) * self.btc_per_input_fee()
    }
}
