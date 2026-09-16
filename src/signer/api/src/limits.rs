//! Size limits for caller-supplied inputs.
//!
//! The signer charges a flat fee for its public key methods, but the cost it pays for a
//! call grows with the number of caller-supplied bytes: the canister pays an ingress
//! induction fee proportional to the size of every message it accepts, and pays again to
//! forward the caller's derivation path and key ID to the management canister.  Without a
//! cap on those bytes, a funded caller can make the signer spend orders of magnitude more
//! cycles than it collects, draining the balance of a service shared by all users.
//!
//! There are two layers.  [`ingress_limit`] bounds the raw message before the
//! signer pays to induct it, but it only applies to ingress; [`check_derivation_path`] and
//! [`check_key_name`] bound what the signer forwards, whatever the message arrived on.

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
/// Their fees differ — `btc_caller_sign` is priced per input, `btc_caller_send` per input
/// and per output, and `eth_sign_transaction` is a flat 37B cycles — but the cheapest is
/// still 37B.  At 128 KiB, ingress induction costs about 689M cycles, so the cap bounds what
/// an unfunded caller can make the signer induct to a small fraction of any fee.  128 KiB is
/// roughly 1,600 Bitcoin UTXOs, far beyond any realistic transaction.
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

/// The maximum number of derivation path elements the management canister accepts.
pub const MAX_MANAGEMENT_DERIVATION_PATH_ELEMENTS: usize = 255;

/// The number of derivation path elements the signer prepends to the caller's path: the
/// schema and the key owner's principal.
pub const DERIVATION_PATH_PREFIX_ELEMENTS: usize = 2;

/// The maximum number of derivation path elements a caller may supply.
///
/// The signer prepends [`DERIVATION_PATH_PREFIX_ELEMENTS`] elements of its own, so the
/// caller gets what is left of the management canister's allowance.
pub const MAX_DERIVATION_PATH_ELEMENTS: usize =
    MAX_MANAGEMENT_DERIVATION_PATH_ELEMENTS - DERIVATION_PATH_PREFIX_ELEMENTS;

/// The maximum total size, in bytes, of a caller-supplied derivation path.
///
/// This is the sum of the lengths of every element; the per-element length is not
/// restricted separately because a single element may legitimately hold a long
/// application name.  4 KiB is far above any legitimate use: the documented pattern is a
/// single application-name element, and even a fully populated BIP32-style path, at four
/// bytes per element, is about 1 KiB.
pub const MAX_DERIVATION_PATH_BYTES: usize = 4096;

/// The maximum length, in bytes, of a caller-supplied threshold key name.
///
/// Production key names (`key_1`, `test_key_1`, `dfx_test_key`) are far shorter; the
/// signer forwards the name verbatim, so it has to be bounded too.
pub const MAX_KEY_NAME_BYTES: usize = 128;

/// Checks that a caller-supplied derivation path is within the documented limits.
///
/// # Errors
/// - If the path has more than [`MAX_DERIVATION_PATH_ELEMENTS`] elements.
/// - If the elements total more than [`MAX_DERIVATION_PATH_BYTES`] bytes.
pub fn check_derivation_path(derivation_path: &[Vec<u8>]) -> Result<(), String> {
    let elements = derivation_path.len();
    if elements > MAX_DERIVATION_PATH_ELEMENTS {
        return Err(format!(
            "derivation path has {elements} elements, the maximum is {MAX_DERIVATION_PATH_ELEMENTS}"
        ));
    }
    let bytes: usize = derivation_path.iter().map(Vec::len).sum();
    if bytes > MAX_DERIVATION_PATH_BYTES {
        return Err(format!(
            "derivation path is {bytes} bytes, the maximum is {MAX_DERIVATION_PATH_BYTES}"
        ));
    }
    Ok(())
}

/// Checks that a caller-supplied threshold key name is within the documented limit.
///
/// # Errors
/// - If the name is longer than [`MAX_KEY_NAME_BYTES`] bytes.
pub fn check_key_name(name: &str) -> Result<(), String> {
    let bytes = name.len();
    if bytes > MAX_KEY_NAME_BYTES {
        return Err(format!(
            "key name is {bytes} bytes, the maximum is {MAX_KEY_NAME_BYTES}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod ingress_tests {
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

#[cfg(test)]
mod forwarding_tests {
    use candid::{Encode, Principal};
    use ic_cdk_management_canister::{SchnorrAlgorithm, SchnorrKeyId, SchnorrPublicKeyArgs};

    use super::{
        check_derivation_path, check_key_name, MAX_DERIVATION_PATH_BYTES,
        MAX_DERIVATION_PATH_ELEMENTS, MAX_KEY_NAME_BYTES, MAX_PUBLIC_KEY_ARG_BYTES,
    };

    /// The largest request the forwarding limits allow still fits inside the ingress limit.
    ///
    /// The two layers are set independently, so a change to either could contradict the
    /// other and start refusing valid calls.  This encodes a request that saturates both
    /// the element count and the byte budget, so the Candid framing and the per-element
    /// length prefixes are counted rather than estimated.
    #[test]
    fn the_ingress_limit_admits_the_largest_permitted_request() {
        let per = MAX_DERIVATION_PATH_BYTES / MAX_DERIVATION_PATH_ELEMENTS;
        let mut path = vec![vec![0u8; per]; MAX_DERIVATION_PATH_ELEMENTS];
        let used: usize = path.iter().map(Vec::len).sum();
        path[0].extend(vec![0u8; MAX_DERIVATION_PATH_BYTES - used]);
        assert_eq!(check_derivation_path(&path), Ok(()), "should be permitted");

        let arg = SchnorrPublicKeyArgs {
            canister_id: Some(Principal::management_canister()),
            derivation_path: path,
            key_id: SchnorrKeyId {
                algorithm: SchnorrAlgorithm::Ed25519,
                name: "k".repeat(MAX_KEY_NAME_BYTES),
            },
        };
        // The second argument is the payment type; `None` is its smallest encoding, and the
        // difference is far below the headroom this assertion leaves.
        let encoded = Encode!(&arg, &Option::<u8>::None).expect("should encode");
        assert!(
            encoded.len() < MAX_PUBLIC_KEY_ARG_BYTES,
            "The largest permitted request encodes to {} bytes, which the {MAX_PUBLIC_KEY_ARG_BYTES} \
             byte ingress limit would refuse.",
            encoded.len()
        );
    }

    #[test]
    fn ordinary_paths_are_accepted() {
        // No path at all.
        assert_eq!(check_derivation_path(&[]), Ok(()));
        // The documented minimum: an application name.
        assert_eq!(check_derivation_path(&[b"MY APP".to_vec()]), Ok(()));
        // A fully populated BIP32-style path.
        let bip32 = vec![vec![0u8; 4]; MAX_DERIVATION_PATH_ELEMENTS];
        assert_eq!(check_derivation_path(&bip32), Ok(()));
    }

    #[test]
    fn limits_are_inclusive() {
        // Exactly at the byte limit, in one element and spread over many.
        assert_eq!(
            check_derivation_path(&[vec![0u8; MAX_DERIVATION_PATH_BYTES]]),
            Ok(())
        );
        let spread = vec![vec![0u8; MAX_DERIVATION_PATH_BYTES / 4]; 4];
        assert_eq!(check_derivation_path(&spread), Ok(()));
        // Exactly at the element limit.
        assert_eq!(
            check_derivation_path(&vec![vec![]; MAX_DERIVATION_PATH_ELEMENTS]),
            Ok(())
        );
        // Exactly at the key name limit.
        assert_eq!(check_key_name(&"k".repeat(MAX_KEY_NAME_BYTES)), Ok(()));
    }

    #[test]
    fn oversized_paths_are_rejected() {
        // One huge element: the shape used by the reported cycle drain.
        assert!(check_derivation_path(&[vec![0u8; MAX_DERIVATION_PATH_BYTES + 1]]).is_err());
        // The same total spread over many elements, so counting elements alone would not
        // catch it.
        let spread = vec![vec![0u8; MAX_DERIVATION_PATH_BYTES / 4 + 1]; 4];
        assert!(check_derivation_path(&spread).is_err());
        // Too many elements, even though they are empty.
        assert!(check_derivation_path(&vec![vec![]; MAX_DERIVATION_PATH_ELEMENTS + 1]).is_err());
    }

    #[test]
    fn oversized_key_names_are_rejected() {
        assert!(check_key_name(&"k".repeat(MAX_KEY_NAME_BYTES + 1)).is_err());
    }
}
