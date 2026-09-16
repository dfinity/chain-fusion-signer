//! Size limits for caller-supplied inputs.
//!
//! The signer charges a flat fee for its public key methods, but the cost it pays for a
//! call grows with the number of caller-supplied bytes: the canister pays an ingress
//! induction fee proportional to the size of every message it accepts, and pays again to
//! forward the caller's derivation path to the management canister.  Without a cap on
//! those bytes, a funded caller can make the signer spend orders of magnitude more cycles
//! than it collects, draining the balance of a service shared by all users.

/// How large a Candid argument a method accepts over ingress.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IngressLimit {
    /// The method accepts at most this many bytes of Candid argument.
    AtMost(usize),
    /// The method intentionally accepts a caller-supplied payload of any size.
    ///
    /// Reserved for methods whose fee is dominated by a threshold signature large enough
    /// to cover a maximum-size message, and where a cap would break callers signing large
    /// messages.
    Unlimited,
    /// Not a known method: refuse it rather than pay to induct it.
    Unknown,
}

/// The maximum Candid argument, in bytes, for a method whose arguments are a principal, an
/// enum, a fixed-size digest, or nothing at all.
///
/// Far above what any of them encode to, leaving room for the payment argument and for
/// fields being added later.
pub const MAX_SMALL_ARG_BYTES: usize = 2 * 1024;

/// The maximum Candid argument, in bytes, for the methods that forward a derivation path.
///
/// Sized from both ends.  It has to be above any legitimate request: the largest request
/// the limits in this module allow encodes to well under 5 KiB.
///
/// It also has to stay below what the signer charges, because a message this filter accepts
/// is inducted and paid for even if the method then rejects it.  On a 34-node fiduciary
/// subnet ingress induction is `(1_200_000 + 2_000 * bytes) * 34 / 13`, so 8 KiB costs about
/// 46M cycles against the 77M flat fee, while 16 KiB would cost about 89M — more than the
/// fee, which would let a caller burn cycles for free.
pub const MAX_PUBLIC_KEY_ARG_BYTES: usize = 8 * 1024;

/// The maximum Candid argument, in bytes, for the transaction-shaped methods.
///
/// These carry a caller-supplied list of transaction inputs and outputs, or EVM call data.
/// They are priced per input and per output, so a large request pays for itself, but the
/// cap still bounds what an unfunded caller can make the signer induct.  128 KiB is roughly
/// 1,600 Bitcoin UTXOs, far beyond any realistic transaction.
pub const MAX_TRANSACTION_ARG_BYTES: usize = 128 * 1024;

/// How large a Candid argument `method` accepts over ingress.
///
/// Every method is listed.  An unrecognised name is [`IngressLimit::Unknown`] and is
/// refused: message dispatch would reject it anyway, and refusing it here means the signer
/// does not pay to induct it first.
#[must_use]
pub fn ingress_limit(method: &str) -> IngressLimit {
    match method {
        // Arbitrarily large caller-supplied message, covered by the signature fee.
        "schnorr_sign" | "eth_personal_sign" => IngressLimit::Unlimited,
        // Forward a caller-supplied derivation path.
        "generic_caller_ecdsa_public_key" | "generic_sign_with_ecdsa" | "schnorr_public_key" => {
            IngressLimit::AtMost(MAX_PUBLIC_KEY_ARG_BYTES)
        }
        // Carry transaction inputs and outputs, or EVM call data.
        "btc_caller_sign" | "btc_caller_send" | "eth_sign_transaction" => {
            IngressLimit::AtMost(MAX_TRANSACTION_ARG_BYTES)
        }
        // A principal, an enum, a hex digest, or nothing.
        "get_canister_status"
        | "config"
        | "http_request"
        | "eth_address"
        | "eth_address_of_caller"
        | "eth_sign_prehash"
        | "btc_caller_address"
        | "btc_caller_balance"
        | "btc_sign_prehash" => IngressLimit::AtMost(MAX_SMALL_ARG_BYTES),
        _ => IngressLimit::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ingress_limit, IngressLimit, MAX_PUBLIC_KEY_ARG_BYTES, MAX_SMALL_ARG_BYTES,
        MAX_TRANSACTION_ARG_BYTES,
    };

    /// The flat-fee public key methods are capped below what they charge.
    #[test]
    fn flat_fee_public_key_methods_are_capped() {
        for method in ["generic_caller_ecdsa_public_key", "schnorr_public_key"] {
            assert_eq!(
                ingress_limit(method),
                IngressLimit::AtMost(MAX_PUBLIC_KEY_ARG_BYTES),
                "{method} charges a flat fee, so its ingress size has to be capped."
            );
        }
    }

    /// Only the methods that intentionally take a large message are uncapped.
    #[test]
    fn only_message_signing_is_unlimited() {
        for method in ["schnorr_sign", "eth_personal_sign"] {
            assert_eq!(ingress_limit(method), IngressLimit::Unlimited, "{method}");
        }
    }

    /// Methods with bounded arguments get a bounded limit, including the unpaid ones.
    ///
    /// `get_canister_status` takes no arguments and takes no payment, so without a cap a
    /// caller could send it a maximum-size blob and make the signer pay to induct it for
    /// nothing.
    #[test]
    fn bounded_methods_are_bounded() {
        for method in [
            "get_canister_status",
            "config",
            "http_request",
            "eth_address",
            "eth_address_of_caller",
            "eth_sign_prehash",
            "btc_caller_address",
            "btc_caller_balance",
            "btc_sign_prehash",
        ] {
            assert_eq!(
                ingress_limit(method),
                IngressLimit::AtMost(MAX_SMALL_ARG_BYTES),
                "{method}"
            );
        }
        for method in ["btc_caller_sign", "btc_caller_send", "eth_sign_transaction"] {
            assert_eq!(
                ingress_limit(method),
                IngressLimit::AtMost(MAX_TRANSACTION_ARG_BYTES),
                "{method}"
            );
        }
    }

    /// An unrecognised method is refused rather than inducted.
    #[test]
    fn unknown_methods_are_refused() {
        for method in ["", "not_a_method", "schnorr_sign_", "Schnorr_sign"] {
            assert_eq!(ingress_limit(method), IngressLimit::Unknown, "{method:?}");
        }
    }

    /// Every method the canister exposes has a limit.
    ///
    /// Refusing unknown methods is only safe if the table is complete: a method added
    /// without a limit would be refused for every caller.  This reads the generated
    /// interface so that omission fails here instead of in production.
    #[test]
    fn every_candid_method_has_a_limit() {
        let did = include_str!("../../canister/signer.did");
        let service = did
            .split_once("service :")
            .expect("signer.did should declare a service")
            .1;
        let mut found = 0;
        for line in service.lines() {
            let line = line.trim();
            if line.starts_with("//") {
                continue;
            }
            // Service entries look like `method_name : (Args) -> (Result);`.
            let Some((name, rest)) = line.split_once(" :") else {
                continue;
            };
            if !rest.trim_start().starts_with('(')
                || name.is_empty()
                || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                continue;
            }
            found += 1;
            assert_ne!(
                ingress_limit(name),
                IngressLimit::Unknown,
                "{name} is in signer.did but has no ingress limit, so it would be refused."
            );
        }
        assert!(
            found > 10,
            "Expected to parse the service methods, found only {found}."
        );
    }
}
