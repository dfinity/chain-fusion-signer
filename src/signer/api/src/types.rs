use candid::{CandidType, Deserialize, Principal};
use std::fmt::Debug;

pub type Timestamp = u64;

#[derive(CandidType, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct InitArg {
    pub ecdsa_key_name: String,
    /// Root of trust for checking canister signatures.
    pub ic_root_key_der: Option<Vec<u8>>,
}

#[derive(CandidType, Deserialize, Clone, Debug, Eq, PartialEq)]
pub enum Arg {
    Init(InitArg),
    Upgrade,
}

#[derive(CandidType, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub ecdsa_key_name: String,
    /// Root of trust for checking canister signatures.
    pub ic_root_key_raw: Option<Vec<u8>>,
}

pub mod transaction {
    use candid::{CandidType, Deserialize, Nat};

    #[derive(CandidType, Deserialize)]
    pub struct SignRequest {
        pub chain_id: Nat,
        pub to: String,
        pub gas: Nat,
        pub max_fee_per_gas: Nat,
        pub max_priority_fee_per_gas: Nat,
        pub value: Nat,
        pub nonce: Nat,
        pub data: Option<String>,
    }
}
