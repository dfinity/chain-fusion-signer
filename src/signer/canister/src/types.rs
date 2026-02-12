use candid::{CandidType, Deserialize};
use ic_chain_fusion_signer_api::types::Config;
use ic_stable_structures::{memory_manager::VirtualMemory, Cell as StableCell, DefaultMemoryImpl};

pub type VMem = VirtualMemory<DefaultMemoryImpl>;
pub type ConfigCell = StableCell<Option<Candid<Config>>, VMem>;

#[derive(Default)]
pub struct Candid<T>(pub T)
where
    T: CandidType + for<'de> Deserialize<'de>;
