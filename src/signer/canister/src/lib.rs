use candid::Principal;
use ic_cdk::api::management_canister::ecdsa::{
    EcdsaPublicKeyArgument, EcdsaPublicKeyResponse, SignWithEcdsaArgument, SignWithEcdsaResponse,
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
            SendBtcResponse,
        },
        eth::{
            EthPersonalSignError, EthPersonalSignRequest, EthPersonalSignResponse,
            EthSignPrehashError, EthSignPrehashRequest, EthSignPrehashResponse,
            EthSignTransactionError, EthSignTransactionRequest, EthSignTransactionResponse,
        },
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

// ////////////////////
// // ETHEREUM UTILS //
// ////////////////////

/// Returns the Ethereum address of a specified user.
///
/// If no user is specified, the caller's address is returned.
///
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn eth_address(
    request: EthAddressRequest,
    payment: Option<PaymentType>,
) -> Result<EthAddressResponse, EthAddressError> {
    let principal = request.principal.unwrap_or_else(ic_cdk::caller);
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
/// # Panics
/// - If the caller is the anonymous user.
#[update(guard = "caller_is_not_anonymous")]
pub async fn eth_address_of_caller(
    payment: Option<PaymentType>,
) -> Result<EthAddressResponse, EthAddressError> {
    let principal = ic_cdk::caller();
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
            let address =
                bitcoin_utils::principal_to_p2wpkh_address(params.network, &ic_cdk::caller())
                    .await
                    .map_err(|msg| GetAddressError::InternalError { msg })?;

            Ok(GetAddressResponse { address })
        }
    }
}

/// Returns the Bitcoin balance of the caller's address.
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
            let address =
                bitcoin_utils::principal_to_p2wpkh_address(params.network, &ic_cdk::caller())
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

/// Creates, signs and sends a BTC transaction from the caller's address.
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
            let principal = ic_cdk::caller();
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
