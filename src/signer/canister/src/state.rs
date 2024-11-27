use std::cell::RefCell;

use candid::Principal;
use ic_chain_fusion_signer_api::types::{Config, InitArg};
use ic_papi_guard::guards::any::{PaymentGuard, VendorPaymentConfig};
use ic_stable_structures::{
    memory_manager::{MemoryId, MemoryManager},
    DefaultMemoryImpl,
};
use lazy_static::lazy_static;

use crate::types::{Candid, ConfigCell};

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
    pub config: ConfigCell,
}

pub fn set_config(arg: InitArg) {
    let config = Config::from(arg);
    mutate_state(|state| {
        state
            .config
            .set(Some(Candid(config)))
            .expect("setting config should succeed");
    });
}

lazy_static! {
    pub static ref PAYMENT_GUARD: PaymentGuard<5> = PaymentGuard {
        supported: [
            VendorPaymentConfig::AttachedCycles,
            VendorPaymentConfig::CallerPaysIcrc2Cycles,
            VendorPaymentConfig::PatronPaysIcrc2Cycles,
            VendorPaymentConfig::CallerPaysIcrc2Tokens {
                ledger: payment_ledger(),
            },
            VendorPaymentConfig::PatronPaysIcrc2Tokens {
                ledger: payment_ledger(),
            },
        ],
    };
}

/// Provides the canister id of the ledger used for payments.
pub fn payment_ledger() -> Principal {
    read_config(|config| config.cycles_ledger)
}
