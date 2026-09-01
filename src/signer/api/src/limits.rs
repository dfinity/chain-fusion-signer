//! Size limits for caller-supplied inputs.
//!
//! The signer charges a flat fee for its public key methods, but the cost it pays for a
//! call grows with the number of caller-supplied bytes: the canister pays an ingress
//! induction fee proportional to the size of every message it accepts, and pays again to
//! forward the caller's derivation path and key ID to the management canister.  Without a
//! cap on those bytes, a funded caller can make the signer spend orders of magnitude more
//! cycles than it collects, draining the balance of a service shared by all users.
//!
//! There are two layers.  [`max_ingress_arg_bytes`] bounds the raw message before the
//! signer pays to induct it, but it only applies to ingress; [`check_derivation_path`] and
//! [`check_key_name`] bound what the signer forwards, whatever the message arrived on.

/// The signer methods that charge a flat fee yet accept caller-controlled bytes.
///
/// The signing methods are not listed: their fee is dominated by the threshold signature
/// cost, which is large enough to cover a maximum-size message.
pub const FLAT_FEE_PUBLIC_KEY_METHODS: [&str; 2] =
    ["generic_caller_ecdsa_public_key", "schnorr_public_key"];

/// The maximum size, in bytes, of the Candid argument the methods in
/// [`FLAT_FEE_PUBLIC_KEY_METHODS`] accept over ingress.
///
/// This is far above any legitimate request: it leaves room for the largest permitted
/// derivation path and key ID plus Candid framing.
pub const MAX_PUBLIC_KEY_ARG_BYTES: usize = 16 * 1024;

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

/// The maximum ingress argument size for `method`, or `None` if the method is not size
/// limited.
#[must_use]
pub fn max_ingress_arg_bytes(method: &str) -> Option<usize> {
    FLAT_FEE_PUBLIC_KEY_METHODS
        .contains(&method)
        .then_some(MAX_PUBLIC_KEY_ARG_BYTES)
}

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
mod tests {
    use super::{
        check_derivation_path, check_key_name, max_ingress_arg_bytes, MAX_DERIVATION_PATH_BYTES,
        MAX_DERIVATION_PATH_ELEMENTS, MAX_KEY_NAME_BYTES, MAX_PUBLIC_KEY_ARG_BYTES,
    };

    #[test]
    fn flat_fee_public_key_methods_are_limited() {
        for method in ["generic_caller_ecdsa_public_key", "schnorr_public_key"] {
            assert_eq!(
                max_ingress_arg_bytes(method),
                Some(MAX_PUBLIC_KEY_ARG_BYTES),
                "{method} charges a flat fee, so its ingress size has to be capped."
            );
        }
    }

    #[test]
    fn other_methods_are_not_limited() {
        // The signing methods pay for themselves out of the threshold signature fee, and
        // capping them would break callers signing large messages.
        for method in [
            "generic_sign_with_ecdsa",
            "schnorr_sign",
            "eth_personal_sign",
            "btc_caller_send",
            "get_canister_status",
        ] {
            assert_eq!(max_ingress_arg_bytes(method), None, "{method}");
        }
    }

    #[test]
    fn the_ingress_limit_admits_the_largest_permitted_path() {
        // A request at the semantic limit has to fit inside the raw ingress limit,
        // otherwise the two layers would contradict each other and reject valid calls.
        assert!(MAX_DERIVATION_PATH_BYTES + MAX_KEY_NAME_BYTES < MAX_PUBLIC_KEY_ARG_BYTES);
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
