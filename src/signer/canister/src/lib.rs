use crate::guards::caller_is_not_anonymous;
use candid::Principal;
use ic_cdk_macros::{export_candid, init, post_upgrade, query, update};
use ic_chain_fusion_signer_api::http::HttpRequest;
use ic_chain_fusion_signer_api::http::HttpResponse;
use ic_chain_fusion_signer_api::metrics::get_metrics;
use ic_chain_fusion_signer_api::std_canister_status;
use ic_chain_fusion_signer_api::types::bitcoin::GetAddressError;
use ic_chain_fusion_signer_api::types::bitcoin::GetAddressRequest;
use ic_chain_fusion_signer_api::types::bitcoin::GetAddressResponse;
use ic_chain_fusion_signer_api::types::bitcoin::GetBalanceRequest;
use ic_chain_fusion_signer_api::types::transaction::SignRequest;
use ic_chain_fusion_signer_api::types::{Arg, Config};
use ic_chain_fusion_signer_api::types::bitcoin::{BitcoinAddressType, GetBalanceError, GetBalanceResponse};
use serde_bytes::ByteBuf;
use sign::bitcoin::{bitcoin_api, bitcoin_utils};
use sign::eth;
use state::{read_config, read_state, set_config};

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

// ////////////////////
// // ETHEREUM UTILS //
// ////////////////////

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
async fn caller_btc_address(params: GetAddressRequest) -> Result<GetAddressResponse, GetAddressError> {
    match params.address_type {
        BitcoinAddressType::P2WPKH => {
            let address = bitcoin_utils::principal_to_p2wpkh_address(
                params.network,
                &ic_cdk::caller(),
            )
            .await
            .map_err(|msg| GetAddressError::InternalError { 
                msg,
            })?;

            Ok(GetAddressResponse { address })
        }
    }
}

/// Returns the Bitcoin balance of the caller's address.
#[update(guard = "caller_is_not_anonymous")]
async fn caller_btc_balance(params: GetBalanceRequest) -> Result<GetBalanceResponse, GetBalanceError> {
    match params.address_type {
        BitcoinAddressType::P2WPKH => {
            let address = bitcoin_utils::principal_to_p2wpkh_address(
                params.network,
                &ic_cdk::caller(),
            )
            .await
            .map_err(|msg| GetBalanceError::InternalError { 
                msg,
            })?;

            let balance = bitcoin_api::get_balance(params.network, address)
                .await
                .map_err(|msg| GetBalanceError::InternalError { msg })?;

            Ok(GetBalanceResponse { balance: balance.into() })
        }
    }
}

// /////////////////////
// // GENERATE CANDID //
// /////////////////////

export_candid!();
