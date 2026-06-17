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
    BtcCallerSendRawTransaction,
    BtcCallerSign,
    SchnorrPublicKey,
    SchnorrSign,
}

impl SignerMethods {
    /// The cost, in cycles, of every paid chain fusion signer API method.
    ///
    /// For most methods this is the exact fee deducted by the canister.
    ///
    /// For `BtcCallerSign` and `BtcCallerSend` the deducted fee depends on the
    /// transaction shape (one `sign_with_ecdsa` per input, and for `BtcCallerSend` the
    /// byte-based `bitcoin_send_transaction` cost, which grows with the number of
    /// outputs). The value returned here is a **grace-period default** sized for a
    /// 2-input, 2-output transaction; it is intended to give existing callers (who
    /// pre-approve a single cycle amount) a soft landing while they migrate. The
    /// canister itself deducts the amount computed by [`Self::btc_fee_for_inputs`] for
    /// `BtcCallerSign`, and by [`Self::btc_fee_for_tx`] for `BtcCallerSend` (with
    /// `n_outputs` including a potential change output). Callers should compute the total
    /// with those helpers rather than relying on `fee()`.
    #[must_use]
    #[allow(clippy::match_same_arms)]
    pub fn fee(&self) -> u128 {
        // Note: Fees are determined with the aid of scripts/check-pricing
        match self {
            SignerMethods::BtcCallerAddress => 79_000_000,
            SignerMethods::BtcCallerBalance => 113_000_000,
            // Grace-period default sized for a 2-input, 2-output transaction:
            // btc_base_fee() + 2 * btc_per_input_fee() + 2 * btc_per_output_fee()
            //   = 95 B + 2 * 37 B + 2 * 1 B = 171 B
            SignerMethods::BtcCallerSend => 171_000_000_000,
            // Flat fee for broadcasting an externally-built, already-signed raw transaction.
            //
            // Unlike `BtcCallerSend`, this method performs NO t-ECDSA signing, so it is NOT
            // priced per input. Its only cost is the `bitcoin_send_transaction` outcall, whose
            // mainnet cost is `5e9 + 20e6 * transaction_bytes`. A raw transaction can be large
            // (the protocol caps it around 100 KB), giving a worst-case outcall cost of roughly
            // `5e9 + 20e6 * 100_000 ≈ 2e12` cycles for a pathological payload, but a few hundred
            // bytes for a normal one.
            //
            // The endpoint is a public relay, so the threat model is DoS / cycle-drain rather
            // than UTXO theft (a valid signature is still required for the Bitcoin network to
            // accept the tx). A flat 100 B cycles sits in the same magnitude family as the other
            // BTC methods, comfortably exceeds the broadcast cost of any realistic transaction,
            // and makes spamming the relay to drain canister cycles uneconomical. It is a flat
            // fee because there is no per-input/per-signature work to meter here.
            SignerMethods::BtcCallerSendRawTransaction => 100_000_000_000,
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
    ///
    /// Note: this prices inputs only. `BtcCallerSend` also pays a byte-based broadcast
    /// cost that grows with the number of outputs; use [`Self::btc_fee_for_tx`] there.
    #[must_use]
    pub fn btc_fee_for_inputs(&self, n_inputs: u64) -> u128 {
        self.btc_base_fee() + u128::from(n_inputs) * self.btc_per_input_fee()
    }

    /// The cost, in cycles, charged per BTC transaction *output*.
    ///
    /// `BtcCallerSend` broadcasts the transaction via `bitcoin_send_transaction`, whose
    /// cost is `5e9 + 20e6 * transaction_bytes` (mainnet). Each output adds a fixed
    /// ~31–43 bytes to the serialized transaction (no witness data), i.e. up to ~860M
    /// cycles; this rounds up to 1e9 to leave margin. `BtcCallerSign` never broadcasts,
    /// so it pays nothing per output.
    #[must_use]
    pub fn btc_per_output_fee(&self) -> u128 {
        match self {
            SignerMethods::BtcCallerSend => 1_000_000_000,
            _ => 0,
        }
    }

    /// Total fee for a BTC send call that processes `n_inputs` inputs and `n_outputs`
    /// outputs (including a potential change output; `btc_caller_send` charges
    /// `requested_outputs + 1`).
    ///
    /// Inputs drive the per-signature (`sign_with_ecdsa`) cost; outputs drive the
    /// byte-based `bitcoin_send_transaction` cost. Pricing both prevents a caller from
    /// inflating the canister's broadcast cost with many (potentially zero-value)
    /// outputs while paying only the input-based fee.
    #[must_use]
    pub fn btc_fee_for_tx(&self, n_inputs: u64, n_outputs: u64) -> u128 {
        self.btc_fee_for_inputs(n_inputs) + u128::from(n_outputs) * self.btc_per_output_fee()
    }
}

#[cfg(test)]
mod tests {
    use super::SignerMethods::{BtcCallerSend, BtcCallerSendRawTransaction, BtcCallerSign};

    const B: u128 = 1_000_000_000;

    #[test]
    fn send_raw_transaction_charges_a_flat_fee() {
        // The raw-broadcast relay does no t-ECDSA signing, so its fee is a single flat value and
        // does not scale with inputs or outputs.
        assert_eq!(BtcCallerSendRawTransaction.fee(), 100 * B);
        assert_eq!(BtcCallerSendRawTransaction.btc_per_input_fee(), 0);
        assert_eq!(BtcCallerSendRawTransaction.btc_per_output_fee(), 0);
        assert_eq!(
            BtcCallerSendRawTransaction.btc_fee_for_tx(10, 10),
            BtcCallerSendRawTransaction.fee(),
        );
    }

    #[test]
    fn send_prices_each_output() {
        // Only BtcCallerSend pays per output; the cost grows linearly with output count.
        assert_eq!(BtcCallerSend.btc_per_output_fee(), B);
        for (n_in, n_out) in [(1, 1), (1, 5_000), (3, 200)] {
            assert_eq!(
                BtcCallerSend.btc_fee_for_tx(n_in, n_out),
                BtcCallerSend.btc_fee_for_inputs(n_in) + u128::from(n_out) * B,
            );
        }
    }

    #[test]
    fn send_grace_default_matches_2in_2out() {
        // The fee() grace-period default is the precise cost of a 2-input, 2-output tx.
        assert_eq!(BtcCallerSend.btc_fee_for_tx(2, 2), BtcCallerSend.fee());
    }

    #[test]
    fn sign_does_not_pay_for_outputs() {
        // BtcCallerSign never broadcasts, so outputs are free and fee_for_tx == fee_for_inputs.
        assert_eq!(BtcCallerSign.btc_per_output_fee(), 0);
        assert_eq!(
            BtcCallerSign.btc_fee_for_tx(2, 10_000),
            BtcCallerSign.btc_fee_for_inputs(2),
        );
    }
}
