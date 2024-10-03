// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use candid::{self, CandidType, Deserialize, Principal};
use ic_cdk::api::call::CallResult as Result;

#[derive(CandidType, Deserialize)]
pub struct InitArg {
  pub ecdsa_key_name: String,
  pub ic_root_key_der: Option<serde_bytes::ByteBuf>,
  pub cycles_ledger: Option<Principal>,
}
#[derive(CandidType, Deserialize)]
pub enum Arg { Upgrade, Init(InitArg) }
#[derive(CandidType, Deserialize)]
pub enum BitcoinNetwork {
  #[serde(rename="mainnet")]
  Mainnet,
  #[serde(rename="regtest")]
  Regtest,
  #[serde(rename="testnet")]
  Testnet,
}
#[derive(CandidType, Deserialize)]
pub enum BitcoinAddressType { #[serde(rename="P2WPKH")] P2Wpkh }
#[derive(CandidType, Deserialize)]
pub struct GetAddressRequest {
  pub network: BitcoinNetwork,
  pub address_type: BitcoinAddressType,
}
#[derive(CandidType, Deserialize)]
pub struct Account {
  pub owner: Principal,
  pub subaccount: Option<serde_bytes::ByteBuf>,
}
#[derive(CandidType, Deserialize)]
pub struct PatronPaysIcrc2Tokens { pub ledger: Principal, pub patron: Account }
#[derive(CandidType, Deserialize)]
pub struct CallerPaysIcrc2Tokens { pub ledger: Principal }
#[derive(CandidType, Deserialize)]
pub enum PaymentType {
  PatronPaysIcrc2Tokens(PatronPaysIcrc2Tokens),
  AttachedCycles,
  CallerPaysIcrc2Cycles,
  CallerPaysIcrc2Tokens(CallerPaysIcrc2Tokens),
  PatronPaysIcrc2Cycles(Account),
}
#[derive(CandidType, Deserialize)]
pub struct GetAddressResponse { pub address: String }
#[derive(CandidType, Deserialize)]
pub enum RejectionCode1 {
  NoError,
  CanisterError,
  SysTransient,
  DestinationInvalid,
  Unknown,
  SysFatal,
  CanisterReject,
}
#[derive(CandidType, Deserialize)]
pub enum WithdrawFromError {
  GenericError{ message: String, error_code: candid::Nat },
  TemporarilyUnavailable,
  InsufficientAllowance{ allowance: candid::Nat },
  Duplicate{ duplicate_of: candid::Nat },
  InvalidReceiver{ receiver: Principal },
  CreatedInFuture{ ledger_time: u64 },
  TooOld,
  FailedToWithdrawFrom{
    withdraw_from_block: Option<candid::Nat>,
    rejection_code: RejectionCode1,
    refund_block: Option<candid::Nat>,
    approval_refund_block: Option<candid::Nat>,
    rejection_reason: String,
  },
  InsufficientFunds{ balance: candid::Nat },
}
#[derive(CandidType, Deserialize)]
pub enum TransferFromError {
  GenericError{ message: String, error_code: candid::Nat },
  TemporarilyUnavailable,
  InsufficientAllowance{ allowance: candid::Nat },
  BadBurn{ min_burn_amount: candid::Nat },
  Duplicate{ duplicate_of: candid::Nat },
  BadFee{ expected_fee: candid::Nat },
  CreatedInFuture{ ledger_time: u64 },
  TooOld,
  InsufficientFunds{ balance: candid::Nat },
}
#[derive(CandidType, Deserialize)]
pub enum PaymentError {
  LedgerWithdrawFromError{ error: WithdrawFromError, ledger: Principal },
  LedgerUnreachable(CallerPaysIcrc2Tokens),
  LedgerTransferFromError{ error: TransferFromError, ledger: Principal },
  UnsupportedPaymentType,
  InsufficientFunds{ needed: u64, available: u64 },
}
#[derive(CandidType, Deserialize)]
pub enum GetAddressError {
  InternalError{ msg: String },
  PaymentError(PaymentError),
}
pub type Result_ = std::result::Result<GetAddressResponse, GetAddressError>;
#[derive(CandidType, Deserialize)]
pub struct GetBalanceResponse { pub balance: u64 }
pub type Result1 = std::result::Result<GetBalanceResponse, GetAddressError>;
#[derive(CandidType, Deserialize)]
pub struct Outpoint { pub txid: serde_bytes::ByteBuf, pub vout: u32 }
#[derive(CandidType, Deserialize)]
pub struct Utxo { pub height: u32, pub value: u64, pub outpoint: Outpoint }
#[derive(CandidType, Deserialize)]
pub struct BtcTxOutput {
  pub destination_address: String,
  pub sent_satoshis: u64,
}
#[derive(CandidType, Deserialize)]
pub struct SendBtcRequest {
  pub fee_satoshis: Option<u64>,
  pub network: BitcoinNetwork,
  pub utxos_to_spend: Vec<Utxo>,
  pub address_type: BitcoinAddressType,
  pub outputs: Vec<BtcTxOutput>,
}
#[derive(CandidType, Deserialize)]
pub struct SendBtcResponse { pub txid: String }
#[derive(CandidType, Deserialize)]
pub enum BuildP2WpkhTxError {
  WrongBitcoinNetwork,
  #[serde(rename="NotP2WPKHSourceAddress")]
  NotP2WpkhSourceAddress,
  InvalidDestinationAddress(GetAddressResponse),
  InvalidSourceAddress(GetAddressResponse),
}
#[derive(CandidType, Deserialize)]
pub enum SendBtcError {
  #[serde(rename="BuildP2wpkhError")]
  BuildP2WpkhError(BuildP2WpkhTxError),
  InternalError{ msg: String },
  PaymentError(PaymentError),
}
pub type Result2 = std::result::Result<SendBtcResponse, SendBtcError>;
#[derive(CandidType, Deserialize)]
pub struct Config {
  pub ecdsa_key_name: String,
  pub ic_root_key_raw: Option<serde_bytes::ByteBuf>,
  pub cycles_ledger: Principal,
}
#[derive(CandidType, Deserialize)]
pub enum GenericSigningError {
  SigningError(RejectionCode1,String,),
  PaymentError(PaymentError),
}
pub type Result3 = std::result::Result<String, GenericSigningError>;
#[derive(CandidType, Deserialize)]
pub struct SignRequest {
  pub to: String,
  pub gas: candid::Nat,
  pub value: candid::Nat,
  pub max_priority_fee_per_gas: candid::Nat,
  pub data: Option<String>,
  pub max_fee_per_gas: candid::Nat,
  pub chain_id: candid::Nat,
  pub nonce: candid::Nat,
}
#[derive(CandidType, Deserialize)]
pub enum EcdsaCurve { #[serde(rename="secp256k1")] Secp256K1 }
#[derive(CandidType, Deserialize)]
pub struct EcdsaKeyId { pub name: String, pub curve: EcdsaCurve }
#[derive(CandidType, Deserialize)]
pub struct EcdsaPublicKeyArgument {
  pub key_id: EcdsaKeyId,
  pub canister_id: Option<Principal>,
  pub derivation_path: Vec<serde_bytes::ByteBuf>,
}
#[derive(CandidType, Deserialize)]
pub struct EcdsaPublicKeyResponse {
  pub public_key: serde_bytes::ByteBuf,
  pub chain_code: serde_bytes::ByteBuf,
}
pub type Result4 = std::result::Result<
  (EcdsaPublicKeyResponse,), GenericSigningError
>;
#[derive(CandidType, Deserialize)]
pub struct SignWithEcdsaArgument {
  pub key_id: EcdsaKeyId,
  pub derivation_path: Vec<serde_bytes::ByteBuf>,
  pub message_hash: serde_bytes::ByteBuf,
}
#[derive(CandidType, Deserialize)]
pub struct SignWithEcdsaResponse { pub signature: serde_bytes::ByteBuf }
pub type Result5 = std::result::Result<
  (SignWithEcdsaResponse,), GenericSigningError
>;
#[derive(CandidType, Deserialize)]
pub enum CanisterStatusType {
  #[serde(rename="stopped")]
  Stopped,
  #[serde(rename="stopping")]
  Stopping,
  #[serde(rename="running")]
  Running,
}
#[derive(CandidType, Deserialize)]
pub struct DefiniteCanisterSettingsArgs {
  pub controller: Principal,
  pub freezing_threshold: candid::Nat,
  pub controllers: Vec<Principal>,
  pub memory_allocation: candid::Nat,
  pub compute_allocation: candid::Nat,
}
#[derive(CandidType, Deserialize)]
pub struct CanisterStatusResultV2 {
  pub controller: Principal,
  pub status: CanisterStatusType,
  pub freezing_threshold: candid::Nat,
  pub balance: Vec<(serde_bytes::ByteBuf,candid::Nat,)>,
  pub memory_size: candid::Nat,
  pub cycles: candid::Nat,
  pub settings: DefiniteCanisterSettingsArgs,
  pub idle_cycles_burned_per_day: candid::Nat,
  pub module_hash: Option<serde_bytes::ByteBuf>,
}
#[derive(CandidType, Deserialize)]
pub struct HttpRequest {
  pub url: String,
  pub method: String,
  pub body: serde_bytes::ByteBuf,
  pub headers: Vec<(String,String,)>,
}
#[derive(CandidType, Deserialize)]
pub struct HttpResponse {
  pub body: serde_bytes::ByteBuf,
  pub headers: Vec<(String,String,)>,
  pub status_code: u16,
}

pub struct Service(pub Principal);
impl Service {
  pub async fn btc_caller_address(&self, arg0: &GetAddressRequest, arg1: &Option<PaymentType>) -> Result<(Result_,)> {
    ic_cdk::call(self.0, "btc_caller_address", (arg0,arg1,)).await
  }
  pub async fn btc_caller_balance(&self, arg0: &GetAddressRequest, arg1: &Option<PaymentType>) -> Result<(Result1,)> {
    ic_cdk::call(self.0, "btc_caller_balance", (arg0,arg1,)).await
  }
  pub async fn btc_caller_send(&self, arg0: &SendBtcRequest, arg1: &Option<PaymentType>) -> Result<(Result2,)> {
    ic_cdk::call(self.0, "btc_caller_send", (arg0,arg1,)).await
  }
  pub async fn caller_eth_address(&self) -> Result<(String,)> {
    ic_cdk::call(self.0, "caller_eth_address", ()).await
  }
  pub async fn config(&self) -> Result<(Config,)> {
    ic_cdk::call(self.0, "config", ()).await
  }
  pub async fn eth_address_of(&self, arg0: &Principal) -> Result<(String,)> {
    ic_cdk::call(self.0, "eth_address_of", (arg0,)).await
  }
  pub async fn eth_address_of_caller(&self, arg0: &Option<PaymentType>) -> Result<(Result3,)> {
    ic_cdk::call(self.0, "eth_address_of_caller", (arg0,)).await
  }
  pub async fn eth_address_of_principal(&self, arg0: &Principal, arg1: &Option<PaymentType>) -> Result<(Result3,)> {
    ic_cdk::call(self.0, "eth_address_of_principal", (arg0,arg1,)).await
  }
  pub async fn eth_personal_sign(&self, arg0: &String, arg1: &Option<PaymentType>) -> Result<(Result3,)> {
    ic_cdk::call(self.0, "eth_personal_sign", (arg0,arg1,)).await
  }
  pub async fn eth_sign_transaction(&self, arg0: &SignRequest, arg1: &Option<PaymentType>) -> Result<(Result3,)> {
    ic_cdk::call(self.0, "eth_sign_transaction", (arg0,arg1,)).await
  }
  pub async fn generic_caller_ecdsa_public_key(&self, arg0: &EcdsaPublicKeyArgument, arg1: &Option<PaymentType>) -> Result<(Result4,)> {
    ic_cdk::call(self.0, "generic_caller_ecdsa_public_key", (arg0,arg1,)).await
  }
  pub async fn generic_sign_with_ecdsa(&self, arg0: &Option<PaymentType>, arg1: &SignWithEcdsaArgument) -> Result<(Result5,)> {
    ic_cdk::call(self.0, "generic_sign_with_ecdsa", (arg0,arg1,)).await
  }
  pub async fn get_canister_status(&self) -> Result<(CanisterStatusResultV2,)> {
    ic_cdk::call(self.0, "get_canister_status", ()).await
  }
  pub async fn http_request(&self, arg0: &HttpRequest) -> Result<(HttpResponse,)> {
    ic_cdk::call(self.0, "http_request", (arg0,)).await
  }
  pub async fn personal_sign(&self, arg0: &String) -> Result<(String,)> {
    ic_cdk::call(self.0, "personal_sign", (arg0,)).await
  }
  pub async fn sign_prehash(&self, arg0: &String) -> Result<(String,)> {
    ic_cdk::call(self.0, "sign_prehash", (arg0,)).await
  }
  pub async fn sign_transaction(&self, arg0: &SignRequest) -> Result<(String,)> {
    ic_cdk::call(self.0, "sign_transaction", (arg0,)).await
  }
}

