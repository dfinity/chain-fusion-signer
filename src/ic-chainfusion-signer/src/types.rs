use candid::{CandidType, Deserialize, Principal};
use ic_stable_structures::{memory_manager::VirtualMemory, Cell as StableCell, DefaultMemoryImpl};
use shared::types::Config;

pub type VMem = VirtualMemory<DefaultMemoryImpl>;
pub type ConfigCell = StableCell<Option<Candid<Config>>, VMem>;

#[derive(Default)]
pub struct Candid<T>(pub T)
where
    T: CandidType + for<'de> Deserialize<'de>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct StoredPrincipal(pub Principal);
