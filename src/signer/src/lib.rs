use crate::bitcoin_utils::public_key_to_p2pkh_address;
use crate::guards::caller_is_not_anonymous;
use candid::{Nat, Principal};
use ethers_core::abi::ethereum_types::{Address, U256, U64};
use ethers_core::types::transaction::eip2930::AccessList;
use ethers_core::types::Bytes;
use ethers_core::utils::keccak256;
use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;

use ic_cdk_macros::{export_candid, init, post_upgrade, query, update};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager},
    DefaultMemoryImpl,
};
use serde_bytes::ByteBuf;
use shared::http::HttpRequest;
use shared::http::HttpResponse;
use shared::metrics::get_metrics;
use shared::std_canister_status;
use shared::types::transaction::SignRequest;
use shared::types::{Arg, Config, InitArg};
use sign::eth;
use std::cell::RefCell;
use types::{Candid, ConfigCell};

mod bitcoin_utils;
mod derivation_path;
mod guards;
mod impls;
mod types;
mod sign;

const CONFIG_MEMORY_ID: MemoryId = MemoryId::new(0);

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static STATE: RefCell<State> = RefCell::new(
        MEMORY_MANAGER.with(|mm| State {
            config: ConfigCell::init(mm.borrow().get(CONFIG_MEMORY_ID), None).expect("config cell initialization should succeed"),
        })
    );
}

pub fn read_state<R>(f: impl FnOnce(&State) -> R) -> R {
    STATE.with(|cell| f(&cell.borrow()))
}

pub fn mutate_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|cell| f(&mut cell.borrow_mut()))
}

/// Reads the internal canister configuration, normally set at canister install or upgrade.
///
/// # Panics
/// - If the `STATE.config` is not initialized.
pub fn read_config<R>(f: impl FnOnce(&Config) -> R) -> R {
    read_state(|state| {
        f(state
            .config
            .get()
            .as_ref()
            .expect("config is not initialized"))
    })
}

pub struct State {
    config: ConfigCell,
}

fn set_config(arg: InitArg) {
    let config = Config::from(arg);
    mutate_state(|state| {
        state
            .config
            .set(Some(Candid(config)))
            .expect("setting config should succeed");
    });
}

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

/// Returns the Bitcoin address of the caller.
#[update(guard = "caller_is_not_anonymous")]
async fn caller_btc_address(network: BitcoinNetwork) -> String {
    public_key_to_p2pkh_address(network, &eth::ecdsa_pubkey_of(&ic_cdk::caller()).await)
}

fn nat_to_u256(n: &Nat) -> U256 {
    let be_bytes = n.0.to_bytes_be();
    U256::from_big_endian(&be_bytes)
}

fn nat_to_u64(n: &Nat) -> U64 {
    let be_bytes = n.0.to_bytes_be();
    U64::from_big_endian(&be_bytes)
}

/// Computes a signature for an [EIP-1559](https://eips.ethereum.org/EIPS/eip-1559) transaction.
#[update(guard = "caller_is_not_anonymous")]
async fn sign_transaction(req: SignRequest) -> String {
    eth::sign_transaction(req).await
}

/// Computes a signature for a hex-encoded message according to [EIP-191](https://eips.ethereum.org/EIPS/eip-191).
#[update(guard = "caller_is_not_anonymous")]
async fn personal_sign(plaintext: String) -> String {
    eth::personal_sign(plaintext).await
}

/// Computes a signature for a precomputed hash.
#[update(guard = "caller_is_not_anonymous")]
async fn sign_prehash(prehash: String) -> String {
    sign::eth::sign_prehash(prehash).await
}

/// API method to get cycle balance and burn rate.
#[update]
async fn get_canister_status() -> std_canister_status::CanisterStatusResultV2 {
    std_canister_status::get_canister_status_v2().await
}

fn decode_hex(hex: &str) -> Bytes {
    Bytes::from(hex::decode(hex.trim_start_matches("0x")).expect("failed to decode hex"))
}

export_candid!();
