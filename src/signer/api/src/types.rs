use candid::{CandidType, Deserialize, Principal};
use std::fmt::Debug;

pub type Timestamp = u64;

#[derive(CandidType, Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct InitArg {
    pub ecdsa_key_name: String,
    /// Root of trust for checking canister signatures.
    pub ic_root_key_der: Option<Vec<u8>>,
    /// Payment canister ID.
    pub cycles_ledger: Option<Principal>,
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
    /// Payment canister ID.
    pub cycles_ledger: Principal,
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

pub mod bitcoin {
    use candid::{CandidType, Deserialize, Nat};
    use ic_cdk::api::management_canister::bitcoin::BitcoinNetwork;

    #[derive(CandidType, Deserialize, Debug)]
    pub enum BitcoinAddressType {
        P2WPKH,
    }

    #[derive(CandidType, Deserialize, Debug)]
    pub struct GetAddressRequest {
        pub network: BitcoinNetwork,
        pub address_type: BitcoinAddressType,
    }

    #[derive(CandidType, Deserialize, Debug)]
    pub struct GetAddressResponse {
        pub address: String,
    }

    #[derive(CandidType, Deserialize, Debug)]
    pub enum GetAddressError {
        InternalError { msg: String },
    }

    #[derive(CandidType, Deserialize, Debug)]
    pub struct GetBalanceRequest {
        pub network: BitcoinNetwork,
        pub address_type: BitcoinAddressType,
    }


    #[derive(CandidType, Deserialize, Debug)]
    pub struct GetBalanceResponse {
        pub balance: Nat,
    }

    #[derive(CandidType, Deserialize, Debug)]
    pub enum GetBalanceError {
        InternalError { msg: String },
    }
}
