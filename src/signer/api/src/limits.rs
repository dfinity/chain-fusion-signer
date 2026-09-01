//! Size limits for caller-supplied inputs.
//!
//! The signer charges a flat fee for its public key methods, but the cost it pays for a
//! call grows with the number of caller-supplied bytes: the canister pays an ingress
//! induction fee proportional to the size of every message it accepts, and pays again to
//! forward the caller's derivation path to the management canister.  Without a cap on
//! those bytes, a funded caller can make the signer spend orders of magnitude more cycles
//! than it collects, draining the balance of a service shared by all users.

/// The signer methods that charge a flat fee yet accept caller-controlled bytes.
///
/// The signing methods are not listed: their fee is dominated by the threshold signature
/// cost, which is large enough to cover a maximum-size message.
pub const FLAT_FEE_PUBLIC_KEY_METHODS: [&str; 2] =
    ["generic_caller_ecdsa_public_key", "schnorr_public_key"];

/// The maximum size, in bytes, of the Candid argument the methods in
/// [`FLAT_FEE_PUBLIC_KEY_METHODS`] accept over ingress.
///
/// This is far above any legitimate request.  The documented derivation path is a single
/// application-name element, and even a fully populated 253-element BIP32-style path is
/// about 1 KiB; the rest is headroom for the key ID and Candid framing.
pub const MAX_PUBLIC_KEY_ARG_BYTES: usize = 16 * 1024;

/// The maximum ingress argument size for `method`, or `None` if the method is not size
/// limited.
#[must_use]
pub fn max_ingress_arg_bytes(method: &str) -> Option<usize> {
    FLAT_FEE_PUBLIC_KEY_METHODS
        .contains(&method)
        .then_some(MAX_PUBLIC_KEY_ARG_BYTES)
}

#[cfg(test)]
mod tests {
    use super::{max_ingress_arg_bytes, MAX_PUBLIC_KEY_ARG_BYTES};

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
}
