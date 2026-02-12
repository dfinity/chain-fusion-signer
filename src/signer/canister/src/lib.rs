use candid::Principal;
use ic_cdk::api::management_canister::{
    ecdsa::{
        EcdsaPublicKeyArgument, EcdsaPublicKeyResponse, SignWithEcdsaArgument,
        SignWithEcdsaResponse,
    },
    schnorr::{
        SchnorrPublicKeyArgument, SchnorrPublicKeyResponse, SignWithSchnorrArgument,
        SignWithSchnorrResponse,
    },
};
use ic_cdk_macros::{export_candid, init, post_upgrade, query, update};
use ic_chain_fusion_signer_api::{
    http::{HttpRequest, HttpResponse},
    methods::SignerMethods,
    metrics::get_metrics,
    std_canister_status,
    types::{
        bitcoin::{
            BitcoinAddressType, GetAddressError, GetAddressRequest, GetAddressResponse,
            GetBalanceError, GetBalanceRequest, GetBalanceResponse, SendBtcError, SendBtcRequest,
            SendBtcResponse, SignBtcResponse,
        },
        eth::{
            EthPersonalSignError, EthPersonalSignRequest, EthPersonalSignResponse,
            EthSignPrehashError, EthSignPrehashRequest, EthSignPrehashResponse,
            EthSignTransactionError, EthSignTransactionRequest, EthSignTransactionResponse,
        },
        schnorr::{SchnorrPublicKeyError, SchnorrSigningError},
        Arg, Config,
    },
};
use ic_papi_api::PaymentType;
use serde_bytes::ByteBuf;
use sign::{
    bitcoin::{
        bitcoin_api, bitcoin_utils,
        fee_utils::calculate_fee,
        tx_utils::{btc_sign_transaction, build_p2wpkh_transaction},
    },
    eth,
    eth::{EthAddressError, EthAddressRequest, EthAddressResponse},
    generic,
    generic::{GenericCallerEcdsaPublicKeyError, GenericSignWithEcdsaError},
};
use state::{read_config, read_state, set_config, PAYMENT_GUARD};

use crate::guards::caller_is_not_anonymous;

mod convert;
mod derivation_path;
mod guards;
mod impls;
mod sign;
mod state;
mod types;

// /////////////////////////
// // CANISTER MANAGEMENT //
// /////////////////////////

/// Initializes state on canister creation
#[init]
pub fn init(arg: Arg) {
    match arg {
        Arg::Init(arg) => set_config(arg),
        Arg::Upgrade => ic_cdk::trap("upgrade args in init"),
    }
}

/// Updates state after canister upgrade
///
/// # Panics
/// - If there is an attempt to upgrade a canister without existing state.  This is most likely an
///   attempt to upgrade a new canister when an installation was intended.
#[post_upgrade]
pub fn post_upgrade(arg: Option<Arg>) {
    match arg {
        Some(Arg::Init(arg)) => set_config(arg),
        _ => {
            read_state(|s| {
                let _ = s.config.get().as_ref().expect(
                    "config is not initialized: reinstall the canister instead of upgrading",
                );
            });
        }
    }
}

/// Show the canister configuration.
#[query(guard = "caller_is_not_anonymous")]
#[must_use]
pub fn config() -> Config {
    read_config(std::clone::Clone::clone)
}

/// Processes external HTTP requests.
#[query]
#[allow(clippy::needless_pass_by_value)]
#[must_use]
pub fn http_request(request: HttpRequest) -> HttpResponse {
    let path = request
        .url
        .split('?')
        .next()
        .unwrap_or_else(|| unreachable!("Even splitting an empty string yields one entry"));
    match path {
        "/metrics" => get_metrics(),
        _ => HttpResponse {
            status_code: 404,
            headers: vec![],
            body: ByteBuf::from(String::from("Not found.")),
        },
    }
}

/// API method to get cycle balance and burn rate.
#[update]
pub async fn get_canister_status() -> std_canister_status::CanisterStatusResultV2 {
    std_canister_status::get_canister_status_v2().await
}

// ////////////////////////
// // GENERIC SIGNATURES //
// ////////////////////////

/// Returns the generic ECDSA public key of the caller.
///
/// Note: This is an exact dual of the canister [`ecdsa_public_key`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-ecdsa_public_key) method.  The argument and response types are also the same.
///
/// # Warnings
/// - The user supplied derivation path is used as-is.  The caller is responsible for ensuring that
///   unintended sub-keys are not requested.
///
/// # Details
/// - Calls `management_canister::ecdsa::ecdsa_public_key(..)`
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
///
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn generic_caller_ecdsa_public_key(
    arg: EcdsaPublicKeyArgument,
    payment: Option<PaymentType>,
) -> Result<(EcdsaPublicKeyResponse,), GenericCallerEcdsaPublicKeyError> {
    PAYMENT_GUARD
        .deduct(
            payment.unwrap_or(PaymentType::AttachedCycles),
            SignerMethods::GenericCallerEcdsaPublicKey.fee(),
        )
        .await?;
    generic::caller_ecdsa_public_key(arg).await
}

/// Signs a message using the caller's ECDSA key.
///
/// Note: This is an exact dual of the canister [`sign_with_ecdsa`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-sign_with_ecdsa) method.  The argument and response types are also the same.
///
/// # Warnings
/// - The user supplied derivation path is used as-is.  The caller is responsible for ensuring that
///   unintended sub-keys are not requested.
///
/// # Details
/// - Calls `management_canister::ecdsa::sign_with_ecdsa(..)`
///   - Costs: See [Fees for the t-ECDSA production key](https://internetcomputer.org/docs/current/references/t-sigs-how-it-works#fees-for-the-t-ecdsa-production-key)
///
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn generic_sign_with_ecdsa(
    payment: Option<PaymentType>,
    arg: SignWithEcdsaArgument,
) -> Result<(SignWithEcdsaResponse,), GenericSignWithEcdsaError> {
    PAYMENT_GUARD
        .deduct(
            payment.unwrap_or(PaymentType::AttachedCycles),
            SignerMethods::GenericSignWithEcdsa.fee(),
        )
        .await?;
    generic::sign_with_ecdsa(arg).await
}

/// Returns the Schnorr public key of the caller or specified principal.
///
/// Note: This is an exact dual of the canister [`schnorr_public_key`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-schnorr_public_key) method.  The argument and response types are also the same.
///
/// # Arguments
/// - `arg`: The same `SchnorrPublicKeyArgument` as the management canister argument.  The semantics
///   are identical but the meaning of the fields in the new context deserve some explanation.
///   - `arg.canister_id`: The principal of the canister or user for which the Chain Fusion Signer
///     has issued the public key.  If `None`, the caller's public key is returned.
///   - `arg.derivation_path`: The derivation path to the public key.  The caller is responsible for
///     ensuring that the derivation path is used to namespace appropriately and to ensure that
///     unintended sub-keys are not requested.  At minimum, it is recommended to use `vec!["NAME OF
///     YOUR APP".into_bytes()]`.  The maximum derivation path length is 254, one less than when
///     calling the management canister.
///   - `arg.key_id`: The ID of the root threshold key to use.  E.g. `key_1` or `test_key_1`.  See <https://internetcomputer.org/docs/current/references/t-sigs-how-it-works#key-derivation>
///     for details.
/// - `payment`: The payment type to use.  If omitted or `None`, it will be assumed that cycles have
///   been attached.
///
/// # Warnings
/// - The user supplied derivation path is used as-is.  The caller is responsible for ensuring that
///   derivation paths are used to namespace appropriately and to ensure that unintended sub-keys
///   are not requested.
/// - It is recommended that, at minimum, the derivation path should be `vec!["NAME OF YOUR
///   APP".into_bytes()]`
///
/// # Details
/// - Calls `management_canister::schnorr::schnorr_public_key(..)`
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
///
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn schnorr_public_key(
    arg: SchnorrPublicKeyArgument,
    payment: Option<PaymentType>,
) -> Result<(SchnorrPublicKeyResponse,), SchnorrPublicKeyError> {
    PAYMENT_GUARD
        .deduct(
            payment.unwrap_or(PaymentType::AttachedCycles),
            SignerMethods::SchnorrPublicKey.fee(),
        )
        .await?;
    generic::schnorr_public_key(arg).await
}

/// Signs a message using the caller's Schnorr key.
///
/// Note: This is an exact dual of the canister [`sign_with_schnorr`](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-sign_with_schnorr) method.  The argument and response types are also the same.
///
/// # Arguments
/// - `arg`: The same `SignWithSchnorrArgument` as the management canister argument.  The semantics
///   are identical but the meaning of the fields in the new context deserve some explanation.
///   - `arg.message`: The data to sign.  Note that if you have a large amount of data, you are
///     probably better off hashing the data and then signing the hash.
///   - `arg.derivation_path`: The derivation path to the public key.  The caller is responsible for
///     ensuring that the derivation path is used to namespace appropriately and to ensure that
///     unintended sub-keys are not requested.  At minimum, it is recommended to use `vec!["NAME OF
///     YOUR APP".into_bytes()]`.  The maximum derivation path length is 254, one less than when
///     calling the management canister.
///   - `arg.key_id`: The ID of the root threshold key to use.  E.g. `key_1` or `test_key_1`.  See <https://internetcomputer.org/docs/current/references/t-sigs-how-it-works#key-derivation>
///     for details.
/// - `payment`: The payment type to use.  If omitted or `None`, it will be assumed that cycles have
///   been attached.
///
/// # Warnings
/// - The user supplied derivation path is used as-is.  The caller is responsible for ensuring that
///   derivation paths are used to namespace appropriately and to ensure that unintended sub-keys
///   are not requested.
/// - It is recommended that, at minimum, the derivation path should be `vec!["NAME OF YOUR
///   APP".into_bytes()]`
///
///  # Details
/// - Calls `management_canister::schnorr::sign_with_schnorr(..)`
///   - Costs: See [Fees for the t-Schnorr production key](https://internetcomputer.org/docs/current/references/t-sigs-how-it-works#fees-for-the-t-schnorr-production-key)
///
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn schnorr_sign(
    arg: SignWithSchnorrArgument,
    payment: Option<PaymentType>,
) -> Result<(SignWithSchnorrResponse,), SchnorrSigningError> {
    PAYMENT_GUARD
        .deduct(
            payment.unwrap_or(PaymentType::AttachedCycles),
            SignerMethods::SchnorrSign.fee(),
        )
        .await?;
    generic::schnorr_sign(arg).await
}

// ////////////////////
// // ETHEREUM UTILS //
// ////////////////////

/// Returns the Ethereum address of a specified user.
///
/// If no user is specified, the caller's address is returned.
///
/// # Details
/// - Gets the specified user's public key with `management_canister::ecdsa::ecdsa_public_key(..)`
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - Converts the public key to an Ethereum address.
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
///
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn eth_address(
    request: EthAddressRequest,
    payment: Option<PaymentType>,
) -> Result<EthAddressResponse, EthAddressError> {
    let principal = request.principal.unwrap_or_else(ic_cdk::api::msg_caller);
    if principal == Principal::anonymous() {
        // TODO: Why trap rather than return an error?
        ic_cdk::trap("Anonymous principal is not authorized");
    }
    PAYMENT_GUARD
        .deduct(
            payment.unwrap_or(PaymentType::AttachedCycles),
            SignerMethods::EthAddress.fee(),
        )
        .await?;
    eth::eth_address(principal).await
}

/// Returns the Ethereum address of the caller.
///
/// # Details
/// - Gets the caller's public key with `management_canister::ecdsa::ecdsa_public_key(..)`
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - Converts the public key to an Ethereum address.
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
///
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn eth_address_of_caller(
    payment: Option<PaymentType>,
) -> Result<EthAddressResponse, EthAddressError> {
    let principal = ic_cdk::api::msg_caller();
    if principal == Principal::anonymous() {
        // TODO: Why trap rather than return an error?
        ic_cdk::trap("Anonymous principal is not authorized");
    }
    PAYMENT_GUARD
        .deduct(
            payment.unwrap_or(PaymentType::AttachedCycles),
            SignerMethods::EthAddressOfCaller.fee(),
        )
        .await?;
    eth::eth_address(principal).await
}

/// Computes an Ethereum signature for an [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559) transaction.
///
/// # Details
/// - Formats the transaction as an `Eip1559TransactionRequest`.
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - Hashes the transaction.
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - Gets the caller's public key with `management_canister::ecdsa::ecdsa_public_key(..)`
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - Signs the transaction with `management_canister::ecdsa::sign_with_ecdsa(..)`
///   - Costs: See [Fees for the t-ECDSA production key](https://internetcomputer.org/docs/current/references/t-sigs-how-it-works#fees-for-the-t-ecdsa-production-key)
///
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn eth_sign_transaction(
    req: EthSignTransactionRequest,
    payment: Option<PaymentType>,
) -> Result<EthSignTransactionResponse, EthSignTransactionError> {
    PAYMENT_GUARD
        .deduct(
            payment.unwrap_or(PaymentType::AttachedCycles),
            SignerMethods::EthSignTransaction.fee(),
        )
        .await?;
    Ok(EthSignTransactionResponse {
        signature: eth::sign_transaction(req.into()).await?,
    })
}

/// Computes an Ethereum signature for a hex-encoded message according to [EIP-191](https://eips.ethereum.org/EIPS/eip-191).
///
/// # Details
/// - Formats the message as `\x19Ethereum Signed Message:\n<length><message>`
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - Hashes the message.
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - Gets the caller's public key with `management_canister::ecdsa::ecdsa_public_key(..)`
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - Signs the message hash with `management_canister::ecdsa::sign_with_ecdsa(..)`
///   - Costs: See [Fees for the t-ECDSA production key](https://internetcomputer.org/docs/current/references/t-sigs-how-it-works#fees-for-the-t-ecdsa-production-key)
///
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn eth_personal_sign(
    request: EthPersonalSignRequest,
    payment: Option<PaymentType>,
) -> Result<EthPersonalSignResponse, EthPersonalSignError> {
    PAYMENT_GUARD
        .deduct(
            payment.unwrap_or(PaymentType::AttachedCycles),
            SignerMethods::EthPersonalSign.fee(),
        )
        .await?;
    Ok(EthPersonalSignResponse {
        signature: eth::personal_sign(request.message).await,
    })
}

/// Computes an Ethereum signature for a precomputed hash.
///
///  # Details
///  Note: This is the same as `eth_personal_sign` but with a precomputed hash, so ingress message
/// size is small regardless of the message length.
///
/// - Gets the caller's public key with `management_canister::ecdsa::ecdsa_public_key(..)`
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - Signs the message hash with `management_canister::ecdsa::sign_with_ecdsa(..)`
///   - Costs: See [Fees for the t-ECDSA production key](https://internetcomputer.org/docs/current/references/t-sigs-how-it-works#fees-for-the-t-ecdsa-production-key)
///
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn eth_sign_prehash(
    req: EthSignPrehashRequest,
    payment: Option<PaymentType>,
) -> Result<EthSignPrehashResponse, EthSignPrehashError> {
    PAYMENT_GUARD
        .deduct(
            payment.unwrap_or(PaymentType::AttachedCycles),
            SignerMethods::EthSignPrehash.fee(),
        )
        .await?;

    Ok(EthSignPrehashResponse {
        signature: eth::sign_prehash(req.hash).await,
    })
}

// ///////////////////
// // BITCOIN UTILS //
// ///////////////////

/// Returns the Bitcoin address of the caller.
///
/// # Details
/// - Gets the principal's public key with `management_canister::ecdsa::ecdsa_public_key(..)`
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - Converts the public key to a P2WPKH address.
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
///
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn btc_caller_address(
    params: GetAddressRequest,
    payment: Option<PaymentType>, /* Note: Do NOT use underscore, please, so that the underscore
                                   * doesn't show up in the generated candid. */
) -> Result<GetAddressResponse, GetAddressError> {
    PAYMENT_GUARD
        .deduct(
            payment.unwrap_or(PaymentType::AttachedCycles),
            SignerMethods::BtcCallerAddress.fee(),
        )
        .await?;
    match params.address_type {
        BitcoinAddressType::P2WPKH => {
            let address = bitcoin_utils::principal_to_p2wpkh_address(
                params.network,
                &ic_cdk::api::msg_caller(),
            )
            .await
            .map_err(|msg| GetAddressError::InternalError { msg })?;

            Ok(GetAddressResponse { address })
        }
    }
}

/// Returns the Bitcoin balance of the caller's address.
///
/// > This method is DEPRECATED. Canister developers are advised to call `bitcoin_get_balance()` on
/// > the Bitcoin (mainnet or testnet) canister.
///
/// # Details
/// - Gets the principal's public key with `management_canister::ecdsa::ecdsa_public_key(..)`
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - Converts the public key to a P2WPKH address.
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - Gets the Bitcoin balance from [the deprecated system Bitcoin API](https://internetcomputer.org/docs/current/references/ic-interface-spec/#ic-bitcoin_get_balance)
///   - Costs: See [Bitcoin API fees and pricing](https://internetcomputer.org/docs/current/references/bitcoin-how-it-works#api-fees-and-pricing)
///
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn btc_caller_balance(
    params: GetBalanceRequest,
    payment: Option<PaymentType>, /* Note: Do NOT use underscore, please, so that the underscore
                                   * doesn't show up in the generated candid. */
) -> Result<GetBalanceResponse, GetBalanceError> {
    PAYMENT_GUARD
        .deduct(
            payment.unwrap_or(PaymentType::AttachedCycles),
            SignerMethods::BtcCallerBalance.fee(),
        )
        .await?;
    match params.address_type {
        BitcoinAddressType::P2WPKH => {
            let address = bitcoin_utils::principal_to_p2wpkh_address(
                params.network,
                &ic_cdk::api::msg_caller(),
            )
            .await
            .map_err(|msg| GetBalanceError::InternalError { msg })?;

            let balance =
                bitcoin_api::get_balance(params.network, address, params.min_confirmations)
                    .await
                    .map_err(|msg| GetBalanceError::InternalError { msg })?;

            Ok(GetBalanceResponse { balance })
        }
    }
}

/// Internal helper that builds and signs a P2WPKH transaction.
async fn sign_btc_transaction_p2wpkh(
    params: &SendBtcRequest,
) -> Result<sign::bitcoin::tx_utils::SignedTransaction, SendBtcError> {
    let principal = ic_cdk::caller();
    let source_address = bitcoin_utils::principal_to_p2wpkh_address(params.network, &principal)
        .await
        .map_err(|msg| SendBtcError::InternalError { msg })?;
    let fee = calculate_fee(
        params.fee_satoshis,
        &params.utxos_to_spend,
        params.network,
        params.outputs.len() as u64,
    )
    .await
    .map_err(|msg| SendBtcError::InternalError { msg })?;

    let transaction = build_p2wpkh_transaction(
        &source_address,
        params.network,
        &params.utxos_to_spend,
        fee,
        &params.outputs,
    )
    .map_err(SendBtcError::BuildP2wpkhError)?;

    btc_sign_transaction(
        &principal,
        transaction,
        &params.utxos_to_spend,
        source_address,
        params.network,
    )
    .await
    .map_err(|msg| SendBtcError::InternalError { msg })
}

/// Creates and signs a BTC transaction from the caller's address without broadcasting it.
///
/// # Details
/// - Gets the principal's public key with `management_canister::ecdsa::ecdsa_public_key(..)`
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - Converts the public key to a P2WPKH address.
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - For every transaction input:
///   - Calls `sign_with_ecdsa(..)` on that input.
///   - Costs: See [Fees for the t-ECDSA production key](https://internetcomputer.org/docs/current/references/t-sigs-how-it-works#fees-for-the-t-ecdsa-production-key)
///
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn btc_caller_sign(
    params: SendBtcRequest,
    payment: Option<PaymentType>,
) -> Result<SignBtcResponse, SendBtcError> {
    PAYMENT_GUARD
        .deduct(
            payment.unwrap_or(PaymentType::AttachedCycles),
            SignerMethods::BtcCallerSign.fee(),
        )
        .await?;
    match params.address_type {
        BitcoinAddressType::P2WPKH => {
            let signed_transaction = sign_btc_transaction_p2wpkh(&params).await?;
            Ok(SignBtcResponse {
                signed_transaction_hex: hex::encode(&signed_transaction.signed_transaction_bytes),
                txid: signed_transaction.txid,
            })
        }
    }
}

/// Creates, signs and sends a BTC transaction from the caller's address.
///
/// # Details
/// - Gets the principal's public key with `management_canister::ecdsa::ecdsa_public_key(..)`
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - Converts the public key to a P2WPKH address.
///   - Costs: [Execution cycles](https://internetcomputer.org/docs/current/developer-docs/gas-cost#execution)
/// - For every transaction input:
///   - Calls `sign_with_ecdsa(..)` on that input.
///   - Costs: See [Fees for the t-ECDSA production key](https://internetcomputer.org/docs/current/references/t-sigs-how-it-works#fees-for-the-t-ecdsa-production-key)
/// - Sends the transaction with `bitcoin_api::send_transaction(..)`
///   - Costs: See [Bitcoin API fees and pricing](https://internetcomputer.org/docs/current/references/bitcoin-how-it-works#api-fees-and-pricing)
///
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn btc_caller_send(
    params: SendBtcRequest,
    payment: Option<PaymentType>,
) -> Result<SendBtcResponse, SendBtcError> {
    PAYMENT_GUARD
        .deduct(
            payment.unwrap_or(PaymentType::AttachedCycles),
            SignerMethods::BtcCallerSend.fee(),
        )
        .await?;
    match params.address_type {
        BitcoinAddressType::P2WPKH => {
            let principal = ic_cdk::api::msg_caller();
            let source_address =
                bitcoin_utils::principal_to_p2wpkh_address(params.network, &principal)
                    .await
                    .map_err(|msg| SendBtcError::InternalError { msg })?;
            let fee = calculate_fee(
                params.fee_satoshis,
                &params.utxos_to_spend,
                params.network,
                params.outputs.len() as u64,
            )
            .await
            .map_err(|msg| SendBtcError::InternalError { msg })?;

            let transaction = build_p2wpkh_transaction(
                &source_address,
                params.network,
                &params.utxos_to_spend,
                fee,
                &params.outputs,
            )
            .map_err(SendBtcError::BuildP2wpkhError)?;

            let signed_transaction = btc_sign_transaction(
                &principal,
                transaction,
                &params.utxos_to_spend,
                source_address.clone(),
                params.network,
            )
            .await
            .map_err(|msg| SendBtcError::InternalError { msg })?;

            bitcoin_api::send_transaction(
                params.network,
                signed_transaction.signed_transaction_bytes,
            )
            .await
            .map_err(|msg| SendBtcError::InternalError { msg })?;

            Ok(SendBtcResponse {
                txid: signed_transaction.txid,
            })
        }
    }
}

// /////////////////////
// // GENERATE CANDID //
// /////////////////////

export_candid!();
