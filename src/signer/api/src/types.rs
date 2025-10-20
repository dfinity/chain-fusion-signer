use std::fmt::Debug;

use candid::{CandidType, Deserialize, Principal};
use ic_cdk::call::RejectCode as IcCdkRejectCode;

/// Local RejectCode type that matches the upstream ic-error-types RejectCode
#[derive(CandidType, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectCode {
    NoError = 0,
    SysFatal = 1,
    SysTransient = 2,
    DestinationInvalid = 3,
    CanisterReject = 4,
    CanisterError = 5,
    Unknown,
}

impl From<IcCdkRejectCode> for RejectCode {
    fn from(code: IcCdkRejectCode) -> Self {
        match code {
            IcCdkRejectCode::NoError => RejectCode::NoError,
            IcCdkRejectCode::SysFatal => RejectCode::SysFatal,
            IcCdkRejectCode::SysTransient => RejectCode::SysTransient,
            IcCdkRejectCode::DestinationInvalid => RejectCode::DestinationInvalid,
            IcCdkRejectCode::CanisterReject => RejectCode::CanisterReject,
            IcCdkRejectCode::CanisterError => RejectCode::CanisterError,
            IcCdkRejectCode::Unknown => RejectCode::Unknown,
        }
    }
}

impl From<RejectCode> for IcCdkRejectCode {
    fn from(code: RejectCode) -> Self {
        match code {
            RejectCode::NoError => IcCdkRejectCode::NoError,
            RejectCode::SysFatal => IcCdkRejectCode::SysFatal,
            RejectCode::SysTransient => IcCdkRejectCode::SysTransient,
            RejectCode::DestinationInvalid => IcCdkRejectCode::DestinationInvalid,
            RejectCode::CanisterReject => IcCdkRejectCode::CanisterReject,
            RejectCode::CanisterError => IcCdkRejectCode::CanisterError,
            RejectCode::Unknown => IcCdkRejectCode::Unknown,
        }
    }
}

pub mod eth;
pub mod generic;
pub mod schnorr;

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
    use candid::{CandidType, Deserialize};
    use ic_cdk::api::management_canister::bitcoin::{BitcoinNetwork, Utxo};
    use ic_papi_api::PaymentError;

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
        PaymentError(PaymentError),
    }

    #[derive(CandidType, Deserialize, Debug)]
    pub struct GetBalanceRequest {
        pub network: BitcoinNetwork,
        pub address_type: BitcoinAddressType,
        pub min_confirmations: Option<u32>,
    }

    #[derive(CandidType, Deserialize, Debug)]
    pub struct GetBalanceResponse {
        pub balance: u64,
    }

    #[derive(CandidType, Deserialize, Debug)]
    pub enum GetBalanceError {
        InternalError { msg: String },
        PaymentError(PaymentError),
    }

    #[derive(CandidType, Deserialize, Debug)]
    pub struct BtcTxOutput {
        pub destination_address: String,
        pub sent_satoshis: u64,
    }

    #[derive(CandidType, Deserialize, Debug)]
    pub struct SendBtcRequest {
        pub network: BitcoinNetwork,
        pub address_type: BitcoinAddressType,
        pub utxos_to_spend: Vec<Utxo>,
        pub fee_satoshis: Option<u64>,
        pub outputs: Vec<BtcTxOutput>,
    }

    #[derive(CandidType, Deserialize, Debug)]
    pub struct SendBtcResponse {
        pub txid: String,
    }

    #[derive(CandidType, Deserialize, Debug)]
    pub enum BuildP2wpkhTxError {
        NotP2WPKHSourceAddress,
        InvalidDestinationAddress { address: String },
        InvalidSourceAddress { address: String },
        WrongBitcoinNetwork,
        NotEnoughFunds { required: u64, available: u64 },
    }

    #[derive(CandidType, Deserialize, Debug)]
    pub enum SendBtcError {
        InternalError { msg: String },
        PaymentError(PaymentError),
        BuildP2wpkhError(BuildP2wpkhTxError),
    }
}
