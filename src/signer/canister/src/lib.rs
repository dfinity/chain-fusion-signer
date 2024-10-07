use crate::guards::caller_is_not_anonymous;
use crate::sign::generic::GenericSigningError;
use candid::Principal;
use ic_cdk::api::management_canister::ecdsa::EcdsaPublicKeyArgument;
use ic_cdk::api::management_canister::ecdsa::EcdsaPublicKeyResponse;
use ic_cdk::api::management_canister::ecdsa::SignWithEcdsaArgument;
use ic_cdk::api::management_canister::ecdsa::SignWithEcdsaResponse;
use ic_cdk_macros::{export_candid, init, post_upgrade, query, update};
use ic_chain_fusion_signer_api::http::HttpRequest;
use ic_chain_fusion_signer_api::http::HttpResponse;
use ic_chain_fusion_signer_api::metrics::get_metrics;
use ic_chain_fusion_signer_api::std_canister_status;
use ic_chain_fusion_signer_api::types::bitcoin::GetAddressError;
use ic_chain_fusion_signer_api::types::bitcoin::GetAddressRequest;
use ic_chain_fusion_signer_api::types::bitcoin::GetAddressResponse;
use ic_chain_fusion_signer_api::types::bitcoin::GetBalanceRequest;
use ic_chain_fusion_signer_api::types::bitcoin::SendBtcError;
use ic_chain_fusion_signer_api::types::bitcoin::SendBtcRequest;
use ic_chain_fusion_signer_api::types::bitcoin::SendBtcResponse;
use ic_chain_fusion_signer_api::types::bitcoin::{
    BitcoinAddressType, GetBalanceError, GetBalanceResponse,
};
use ic_chain_fusion_signer_api::types::transaction::SignRequest;
use ic_chain_fusion_signer_api::types::{Arg, Config};
use ic_papi_api::PaymentType;
use ic_papi_guard::guards::{PaymentContext, PaymentGuard2};
use serde_bytes::ByteBuf;
use sign::bitcoin::fee_utils::calculate_fee;
use sign::bitcoin::tx_utils::btc_sign_transaction;
use sign::bitcoin::tx_utils::build_p2wpkh_transaction;
use sign::bitcoin::{bitcoin_api, bitcoin_utils};
use sign::eth;
use sign::generic;
use sign::generic::GenericCallerEcdsaPublicKeyError;
use sign::generic::GenericSignWithEcdsaError;
use state::{read_config, read_state, set_config, PAYMENT_GUARD};

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

#[init]
fn init(arg: Arg) {
    match arg {
        Arg::Init(arg) => set_config(arg),
        Arg::Upgrade => ic_cdk::trap("upgrade args in init"),
    }
}

#[post_upgrade]
fn post_upgrade(arg: Option<Arg>) {
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
fn config() -> Config {
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
async fn get_canister_status() -> std_canister_status::CanisterStatusResultV2 {
    std_canister_status::get_canister_status_v2().await
}

// ////////////////////////
// // GENERIC SIGNATURES //
// ////////////////////////

/// Returns the generic Ed25519 public key of the caller.
///
/// Note: This is an exact dual of the canister `ecdsa_public_key` method.  The argument and response types are also the same.
#[update(guard = "caller_is_not_anonymous")]
async fn generic_caller_ecdsa_public_key(
    arg: EcdsaPublicKeyArgument,
    payment: Option<PaymentType>,
) -> Result<(EcdsaPublicKeyResponse,), GenericCallerEcdsaPublicKeyError> {
    let fee = 1_000_000_000;
    PAYMENT_GUARD
        .deduct(
            PaymentContext::default(),
            payment.unwrap_or(PaymentType::AttachedCycles),
            fee,
        )
        .await?;
    generic::caller_ecdsa_public_key(arg).await
}

/// Returns the generic Ed25519 public key of the caller.
#[update(guard = "caller_is_not_anonymous")]
async fn generic_sign_with_ecdsa(
    payment: Option<PaymentType>,
    arg: SignWithEcdsaArgument,
) -> Result<(SignWithEcdsaResponse,), GenericSignWithEcdsaError> {
    let fee = 1_000_000_000;
    PAYMENT_GUARD
        .deduct(
            PaymentContext::default(),
            payment.unwrap_or(PaymentType::AttachedCycles),
            fee,
        )
        .await?;
    generic::sign_with_ecdsa(arg).await
}

// ////////////////////
// // ETHEREUM UTILS //
// ////////////////////

/// Returns the Ethereum address of the caller.
#[update(guard = "caller_is_not_anonymous")]
async fn eth_address_of_caller(
    payment: Option<PaymentType>,
) -> Result<String, GenericSigningError> {
    PAYMENT_GUARD
        .deduct(
            PaymentContext::default(),
            payment.unwrap_or(PaymentType::AttachedCycles),
            1_000_000_000,
        )
        .await?;
    Ok(eth::pubkey_bytes_to_address(
        &eth::ecdsa_pubkey_of(&ic_cdk::caller()).await,
    ))
}

/// Returns the Ethereum address of the specified user.
#[update(guard = "caller_is_not_anonymous")]
async fn eth_address_of_principal(
    p: Principal,
    payment: Option<PaymentType>,
) -> Result<String, GenericSigningError> {
    if p == Principal::anonymous() {
        ic_cdk::trap("Anonymous principal is not authorized");
    }
    PAYMENT_GUARD
        .deduct(
            PaymentContext::default(),
            payment.unwrap_or(PaymentType::AttachedCycles),
            1_000_000_000,
        )
        .await?;
    Ok(eth::pubkey_bytes_to_address(
        &eth::ecdsa_pubkey_of(&p).await,
    ))
}

/// Computes an Ethereum signature for an [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559) transaction.
#[update(guard = "caller_is_not_anonymous")]
async fn eth_sign_transaction(
    req: SignRequest,
    payment: Option<PaymentType>,
) -> Result<String, GenericSigningError> {
    PAYMENT_GUARD
        .deduct(
            PaymentContext::default(),
            payment.unwrap_or(PaymentType::AttachedCycles),
            1_000_000_000,
        )
        .await?;
    Ok(eth::sign_transaction(req).await)
}

/// Computes an Ethereum signature for a hex-encoded message according to [EIP-191](https://eips.ethereum.org/EIPS/eip-191).
#[update(guard = "caller_is_not_anonymous")]
async fn eth_personal_sign(
    plaintext: String,
    payment: Option<PaymentType>,
) -> Result<String, GenericSigningError> {
    PAYMENT_GUARD
        .deduct(
            PaymentContext::default(),
            payment.unwrap_or(PaymentType::AttachedCycles),
            1_000_000_000,
        )
        .await?;
    Ok(eth::personal_sign(plaintext).await)
}

/// Returns the Ethereum address of the caller.
#[update(guard = "caller_is_not_anonymous")]
async fn caller_eth_address() -> String {
    eth::pubkey_bytes_to_address(&eth::ecdsa_pubkey_of(&ic_cdk::caller()).await)
}

/// Returns the Ethereum address of the specified user.
#[update(guard = "caller_is_not_anonymous")]
async fn eth_address_of(p: Principal) -> String {
    if p == Principal::anonymous() {
        ic_cdk::trap("Anonymous principal is not authorized");
    }
    eth::pubkey_bytes_to_address(&eth::ecdsa_pubkey_of(&p).await)
}

/// Computes an Ethereum signature for an [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559) transaction.
#[update(guard = "caller_is_not_anonymous")]
async fn sign_transaction(req: SignRequest) -> String {
    eth::sign_transaction(req).await
}

/// Computes an Ethereum signature for a hex-encoded message according to [EIP-191](https://eips.ethereum.org/EIPS/eip-191).
#[update(guard = "caller_is_not_anonymous")]
async fn personal_sign(plaintext: String) -> String {
    eth::personal_sign(plaintext).await
}

/// Computes an Ethereum signature for a precomputed hash.
#[update(guard = "caller_is_not_anonymous")]
async fn sign_prehash(prehash: String) -> String {
    eth::sign_prehash(prehash).await
}

// ///////////////////
// // BITCOIN UTILS //
// ///////////////////

/// Returns the Bitcoin address of the caller.
#[update(guard = "caller_is_not_anonymous")]
#[allow(unused_variables)] // TODO: Remove this once the payment guard is used.
async fn btc_caller_address(
    params: GetAddressRequest,
    payment: Option<PaymentType>, // Note: Do NOT use underscore, please, so that the underscore doesn't show up in the generated candid.
) -> Result<GetAddressResponse, GetAddressError> {
    /* TODO: uncomment when the payment guard is ready
    PAYMENT_GUARD
        .deduct(
            PaymentContext::default(),
            payment.unwrap_or(PaymentType::AttachedCycles),
            1_000_000_000,
        )
        .await?;
    */
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
#[update(guard = "caller_is_not_anonymous")]
#[allow(unused_variables)] // TODO: Remove this once the payment guard is used.
async fn btc_caller_balance(
    params: GetBalanceRequest,
    payment: Option<PaymentType>, // Note: Do NOT use underscore, please, so that the underscore doesn't show up in the generated candid.
) -> Result<GetBalanceResponse, GetBalanceError> {
    /* TODO: Uncomment the payment guard once the payment is implemented.
    PAYMENT_GUARD
        .deduct(
            PaymentContext::default(),
            payment.unwrap_or(PaymentType::AttachedCycles),
            1_000_000_000,
        )
        .await?;
    */
    match params.address_type {
        BitcoinAddressType::P2WPKH => {
            let address =
                bitcoin_utils::principal_to_p2wpkh_address(params.network, &ic_cdk::caller())
                    .await
                    .map_err(|msg| GetBalanceError::InternalError { msg })?;

            let balance = bitcoin_api::get_balance(params.network, address)
                .await
                .map_err(|msg| GetBalanceError::InternalError { msg })?;

            Ok(GetBalanceResponse { balance })
        }
    }
}

/// Creates, signs and sends a BTC transaction from the caller's address.
#[update(guard = "caller_is_not_anonymous")]
#[allow(unused_variables)] // TODO: Remove this once the payment guard is used.
async fn btc_caller_send(
    params: SendBtcRequest,
    payment: Option<PaymentType>,
) -> Result<SendBtcResponse, SendBtcError> {
    /* TODO: Uncomment the payment guard once the payment is implemented.
    PAYMENT_GUARD
        .deduct(
            PaymentContext::default(),
            payment.unwrap_or(PaymentType::AttachedCycles),
            1_000_000_000,
        )
        .await?;
    */
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
