// This is an experimental feature to generate Rust binding from Candid.
// You may want to manually adjust some of the types.
#![allow(dead_code, unused_imports)]
use std::sync::Arc;

use candid::{self, decode_one, encode_args, encode_one, CandidType, Deserialize, Principal};
use ic_cdk::api::call::CallResult;
use pocket_ic::{PocketIc, WasmResult};

use super::pic_canister::{PicCanister, PicCanisterTrait};

#[derive(CandidType, Deserialize, Debug)]
pub enum ChangeIndexId {
    SetTo(Principal),
    Unset,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct UpgradeArgs {
    pub change_index_id: Option<ChangeIndexId>,
    pub max_blocks_per_request: Option<u64>,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct InitArgs {
    pub index_id: Option<Principal>,
    pub max_blocks_per_request: u64,
}
#[derive(CandidType, Deserialize, Debug)]
pub enum LedgerArgs {
    Upgrade(Option<UpgradeArgs>),
    Init(InitArgs),
}
#[derive(CandidType, Deserialize, Debug)]
pub struct SubnetFilter {
    pub subnet_type: Option<String>,
}
#[derive(CandidType, Deserialize, Debug)]
pub enum SubnetSelection {
    Filter(SubnetFilter),
    Subnet { subnet: Principal },
}
#[derive(CandidType, Deserialize, Debug)]
pub struct CanisterSettings {
    pub freezing_threshold: Option<candid::Nat>,
    pub controllers: Option<Vec<Principal>>,
    pub reserved_cycles_limit: Option<candid::Nat>,
    pub memory_allocation: Option<candid::Nat>,
    pub compute_allocation: Option<candid::Nat>,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct CmcCreateCanisterArgs {
    pub subnet_selection: Option<SubnetSelection>,
    pub settings: Option<CanisterSettings>,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct CreateCanisterArgs {
    pub from_subaccount: Option<serde_bytes::ByteBuf>,
    pub created_at_time: Option<u64>,
    pub amount: candid::Nat,
    pub creation_args: Option<CmcCreateCanisterArgs>,
}
pub type BlockIndex = candid::Nat;
#[derive(CandidType, Deserialize, Debug)]
pub struct CreateCanisterSuccess {
    pub block_id: BlockIndex,
    pub canister_id: Principal,
}
#[derive(CandidType, Deserialize, Debug)]
pub enum CreateCanisterError {
    GenericError {
        message: String,
        error_code: candid::Nat,
    },
    TemporarilyUnavailable,
    Duplicate {
        duplicate_of: candid::Nat,
        canister_id: Option<Principal>,
    },
    CreatedInFuture {
        ledger_time: u64,
    },
    FailedToCreate {
        error: String,
        refund_block: Option<BlockIndex>,
        fee_block: Option<BlockIndex>,
    },
    TooOld,
    InsufficientFunds {
        balance: candid::Nat,
    },
}
#[derive(CandidType, Deserialize, Debug)]
pub struct Account {
    pub owner: Principal,
    pub subaccount: Option<serde_bytes::ByteBuf>,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct CreateCanisterFromArgs {
    pub spender_subaccount: Option<serde_bytes::ByteBuf>,
    pub from: Account,
    pub created_at_time: Option<u64>,
    pub amount: candid::Nat,
    pub creation_args: Option<CmcCreateCanisterArgs>,
}
#[derive(CandidType, Deserialize, Debug)]
pub enum RejectionCode {
    NoError,
    CanisterError,
    SysTransient,
    DestinationInvalid,
    Unknown,
    SysFatal,
    CanisterReject,
}
#[derive(CandidType, Deserialize, Debug)]
pub enum CreateCanisterFromError {
    FailedToCreateFrom {
        create_from_block: Option<BlockIndex>,
        rejection_code: RejectionCode,
        refund_block: Option<BlockIndex>,
        approval_refund_block: Option<BlockIndex>,
        rejection_reason: String,
    },
    GenericError {
        message: String,
        error_code: candid::Nat,
    },
    TemporarilyUnavailable,
    InsufficientAllowance {
        allowance: candid::Nat,
    },
    Duplicate {
        duplicate_of: candid::Nat,
        canister_id: Option<Principal>,
    },
    CreatedInFuture {
        ledger_time: u64,
    },
    TooOld,
    InsufficientFunds {
        balance: candid::Nat,
    },
}
#[derive(CandidType, Deserialize, Debug)]
pub struct DepositArgs {
    pub to: Account,
    pub memo: Option<serde_bytes::ByteBuf>,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct DepositResult {
    pub balance: candid::Nat,
    pub block_index: BlockIndex,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct HttpRequest {
    pub url: String,
    pub method: String,
    pub body: serde_bytes::ByteBuf,
    pub headers: Vec<(String, String)>,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct HttpResponse {
    pub body: serde_bytes::ByteBuf,
    pub headers: Vec<(String, String)>,
    pub status_code: u16,
}
#[derive(CandidType, Deserialize, Debug)]
pub enum MetadataValue {
    Int(candid::Int),
    Nat(candid::Nat),
    Blob(serde_bytes::ByteBuf),
    Text(String),
}
#[derive(CandidType, Deserialize, Debug)]
pub struct SupportedStandard {
    pub url: String,
    pub name: String,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct TransferArgs {
    pub to: Account,
    pub fee: Option<candid::Nat>,
    pub memo: Option<serde_bytes::ByteBuf>,
    pub from_subaccount: Option<serde_bytes::ByteBuf>,
    pub created_at_time: Option<u64>,
    pub amount: candid::Nat,
}
#[derive(CandidType, Deserialize, Debug)]
pub enum TransferError {
    GenericError {
        message: String,
        error_code: candid::Nat,
    },
    TemporarilyUnavailable,
    BadBurn {
        min_burn_amount: candid::Nat,
    },
    Duplicate {
        duplicate_of: candid::Nat,
    },
    BadFee {
        expected_fee: candid::Nat,
    },
    CreatedInFuture {
        ledger_time: u64,
    },
    TooOld,
    InsufficientFunds {
        balance: candid::Nat,
    },
}
#[derive(CandidType, Deserialize, Debug)]
pub struct AllowanceArgs {
    pub account: Account,
    pub spender: Account,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct Allowance {
    pub allowance: candid::Nat,
    pub expires_at: Option<u64>,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct ApproveArgs {
    pub fee: Option<candid::Nat>,
    pub memo: Option<serde_bytes::ByteBuf>,
    pub from_subaccount: Option<serde_bytes::ByteBuf>,
    pub created_at_time: Option<u64>,
    pub amount: candid::Nat,
    pub expected_allowance: Option<candid::Nat>,
    pub expires_at: Option<u64>,
    pub spender: Account,
}
impl Default for ApproveArgs {
    fn default() -> Self {
        Self {
            fee: None,
            memo: None,
            from_subaccount: None,
            created_at_time: None,
            amount: candid::Nat::from(0u32),
            expected_allowance: None,
            expires_at: None,
            spender: Account {
                owner: Principal::anonymous(),
                subaccount: None,
            },
        }
    }
}
#[derive(CandidType, Deserialize, Debug)]
pub enum ApproveError {
    GenericError {
        message: String,
        error_code: candid::Nat,
    },
    TemporarilyUnavailable,
    Duplicate {
        duplicate_of: candid::Nat,
    },
    BadFee {
        expected_fee: candid::Nat,
    },
    AllowanceChanged {
        current_allowance: candid::Nat,
    },
    CreatedInFuture {
        ledger_time: u64,
    },
    TooOld,
    Expired {
        ledger_time: u64,
    },
    InsufficientFunds {
        balance: candid::Nat,
    },
}
#[derive(CandidType, Deserialize, Debug)]
pub struct TransferFromArgs {
    pub to: Account,
    pub fee: Option<candid::Nat>,
    pub spender_subaccount: Option<serde_bytes::ByteBuf>,
    pub from: Account,
    pub memo: Option<serde_bytes::ByteBuf>,
    pub created_at_time: Option<u64>,
    pub amount: candid::Nat,
}
#[derive(CandidType, Deserialize, Debug)]
pub enum TransferFromError {
    GenericError {
        message: String,
        error_code: candid::Nat,
    },
    TemporarilyUnavailable,
    InsufficientAllowance {
        allowance: candid::Nat,
    },
    BadBurn {
        min_burn_amount: candid::Nat,
    },
    Duplicate {
        duplicate_of: candid::Nat,
    },
    BadFee {
        expected_fee: candid::Nat,
    },
    CreatedInFuture {
        ledger_time: u64,
    },
    TooOld,
    InsufficientFunds {
        balance: candid::Nat,
    },
}
#[derive(CandidType, Deserialize, Debug)]
pub struct GetArchivesArgs {
    pub from: Option<Principal>,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct GetArchivesResultItem {
    pub end: candid::Nat,
    pub canister_id: Principal,
    pub start: candid::Nat,
}
pub type GetArchivesResult = Vec<GetArchivesResultItem>;
#[derive(CandidType, Deserialize, Debug)]
pub struct GetBlocksArgsItem {
    pub start: candid::Nat,
    pub length: candid::Nat,
}
pub type GetBlocksArgs = Vec<GetBlocksArgsItem>;
#[derive(CandidType, Deserialize, Debug)]
pub enum Value {
    Int(candid::Int),
    Map(Vec<(String, Box<Value>)>),
    Nat(candid::Nat),
    Nat64(u64),
    Blob(serde_bytes::ByteBuf),
    Text(String),
    Array(Vec<Box<Value>>),
}
#[derive(CandidType, Deserialize, Debug)]
pub struct GetBlocksResultBlocksItem {
    pub id: candid::Nat,
    pub block: Box<Value>,
}
candid::define_function!(pub GetBlocksResultArchivedBlocksItemCallback : (
    GetBlocksArgs,
  ) -> (GetBlocksResult) query);
#[derive(CandidType, Deserialize, Debug)]
pub struct GetBlocksResultArchivedBlocksItem {
    pub args: GetBlocksArgs,
    pub callback: GetBlocksResultArchivedBlocksItemCallback,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct GetBlocksResult {
    pub log_length: candid::Nat,
    pub blocks: Vec<GetBlocksResultBlocksItem>,
    pub archived_blocks: Vec<GetBlocksResultArchivedBlocksItem>,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct DataCertificate {
    pub certificate: serde_bytes::ByteBuf,
    pub hash_tree: serde_bytes::ByteBuf,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct SupportedBlockType {
    pub url: String,
    pub block_type: String,
}
#[derive(CandidType, Deserialize, Debug)]
pub struct WithdrawArgs {
    pub to: Principal,
    pub from_subaccount: Option<serde_bytes::ByteBuf>,
    pub created_at_time: Option<u64>,
    pub amount: candid::Nat,
}
#[derive(CandidType, Deserialize, Debug)]
pub enum WithdrawError {
    FailedToWithdraw {
        rejection_code: RejectionCode,
        fee_block: Option<candid::Nat>,
        rejection_reason: String,
    },
    GenericError {
        message: String,
        error_code: candid::Nat,
    },
    TemporarilyUnavailable,
    Duplicate {
        duplicate_of: candid::Nat,
    },
    BadFee {
        expected_fee: candid::Nat,
    },
    InvalidReceiver {
        receiver: Principal,
    },
    CreatedInFuture {
        ledger_time: u64,
    },
    TooOld,
    InsufficientFunds {
        balance: candid::Nat,
    },
}
#[derive(CandidType, Deserialize, Debug)]
pub struct WithdrawFromArgs {
    pub to: Principal,
    pub spender_subaccount: Option<serde_bytes::ByteBuf>,
    pub from: Account,
    pub created_at_time: Option<u64>,
    pub amount: candid::Nat,
}
#[derive(CandidType, Deserialize, Debug)]
pub enum WithdrawFromError {
    GenericError {
        message: String,
        error_code: candid::Nat,
    },
    TemporarilyUnavailable,
    InsufficientAllowance {
        allowance: candid::Nat,
    },
    Duplicate {
        duplicate_of: BlockIndex,
    },
    InvalidReceiver {
        receiver: Principal,
    },
    CreatedInFuture {
        ledger_time: u64,
    },
    TooOld,
    FailedToWithdrawFrom {
        withdraw_from_block: Option<candid::Nat>,
        rejection_code: RejectionCode,
        refund_block: Option<candid::Nat>,
        approval_refund_block: Option<candid::Nat>,
        rejection_reason: String,
    },
    InsufficientFunds {
        balance: candid::Nat,
    },
}

pub struct Service(pub Principal);
impl Service {
    pub async fn create_canister(
        &self,
        arg0: &CreateCanisterArgs,
    ) -> CallResult<(std::result::Result<CreateCanisterSuccess, CreateCanisterError>,)> {
        ic_cdk::call(self.0, "create_canister", (arg0,)).await
    }
    pub async fn create_canister_from(
        &self,
        arg0: &CreateCanisterFromArgs,
    ) -> CallResult<(std::result::Result<CreateCanisterSuccess, CreateCanisterFromError>,)> {
        ic_cdk::call(self.0, "create_canister_from", (arg0,)).await
    }
    pub async fn deposit(&self, arg0: &DepositArgs) -> CallResult<(DepositResult,)> {
        ic_cdk::call(self.0, "deposit", (arg0,)).await
    }
    pub async fn http_request(&self, arg0: &HttpRequest) -> CallResult<(HttpResponse,)> {
        ic_cdk::call(self.0, "http_request", (arg0,)).await
    }
    pub async fn icrc_1_balance_of(&self, arg0: &Account) -> CallResult<(candid::Nat,)> {
        ic_cdk::call(self.0, "icrc1_balance_of", (arg0,)).await
    }
    pub async fn icrc_1_decimals(&self) -> CallResult<(u8,)> {
        ic_cdk::call(self.0, "icrc1_decimals", ()).await
    }
    pub async fn icrc_1_fee(&self) -> CallResult<(candid::Nat,)> {
        ic_cdk::call(self.0, "icrc1_fee", ()).await
    }
    pub async fn icrc_1_metadata(&self) -> CallResult<(Vec<(String, MetadataValue)>,)> {
        ic_cdk::call(self.0, "icrc1_metadata", ()).await
    }
    pub async fn icrc_1_minting_account(&self) -> CallResult<(Option<Account>,)> {
        ic_cdk::call(self.0, "icrc1_minting_account", ()).await
    }
    pub async fn icrc_1_name(&self) -> CallResult<(String,)> {
        ic_cdk::call(self.0, "icrc1_name", ()).await
    }
    pub async fn icrc_1_supported_standards(&self) -> CallResult<(Vec<SupportedStandard>,)> {
        ic_cdk::call(self.0, "icrc1_supported_standards", ()).await
    }
    pub async fn icrc_1_symbol(&self) -> CallResult<(String,)> {
        ic_cdk::call(self.0, "icrc1_symbol", ()).await
    }
    pub async fn icrc_1_total_supply(&self) -> CallResult<(candid::Nat,)> {
        ic_cdk::call(self.0, "icrc1_total_supply", ()).await
    }
    pub async fn icrc_1_transfer(
        &self,
        arg0: &TransferArgs,
    ) -> CallResult<(std::result::Result<BlockIndex, TransferError>,)> {
        ic_cdk::call(self.0, "icrc1_transfer", (arg0,)).await
    }
    pub async fn icrc_2_allowance(&self, arg0: &AllowanceArgs) -> CallResult<(Allowance,)> {
        ic_cdk::call(self.0, "icrc2_allowance", (arg0,)).await
    }
    pub async fn icrc_2_approve(
        &self,
        arg0: &ApproveArgs,
    ) -> CallResult<(std::result::Result<candid::Nat, ApproveError>,)> {
        ic_cdk::call(self.0, "icrc2_approve", (arg0,)).await
    }
    pub async fn icrc_2_transfer_from(
        &self,
        arg0: &TransferFromArgs,
    ) -> CallResult<(std::result::Result<candid::Nat, TransferFromError>,)> {
        ic_cdk::call(self.0, "icrc2_transfer_from", (arg0,)).await
    }
    pub async fn icrc_3_get_archives(
        &self,
        arg0: &GetArchivesArgs,
    ) -> CallResult<(GetArchivesResult,)> {
        ic_cdk::call(self.0, "icrc3_get_archives", (arg0,)).await
    }
    pub async fn icrc_3_get_blocks(&self, arg0: &GetBlocksArgs) -> CallResult<(GetBlocksResult,)> {
        ic_cdk::call(self.0, "icrc3_get_blocks", (arg0,)).await
    }
    pub async fn icrc_3_get_tip_certificate(&self) -> CallResult<(Option<DataCertificate>,)> {
        ic_cdk::call(self.0, "icrc3_get_tip_certificate", ()).await
    }
    pub async fn icrc_3_supported_block_types(&self) -> CallResult<(Vec<SupportedBlockType>,)> {
        ic_cdk::call(self.0, "icrc3_supported_block_types", ()).await
    }
    pub async fn withdraw(
        &self,
        arg0: &WithdrawArgs,
    ) -> CallResult<(std::result::Result<BlockIndex, WithdrawError>,)> {
        ic_cdk::call(self.0, "withdraw", (arg0,)).await
    }
    pub async fn withdraw_from(
        &self,
        arg0: &WithdrawFromArgs,
    ) -> CallResult<(std::result::Result<BlockIndex, WithdrawFromError>,)> {
        ic_cdk::call(self.0, "withdraw_from", (arg0,)).await
    }
}

pub struct CyclesLedgerPic {
    pub pic: Arc<PocketIc>,
    pub canister_id: Principal,
}

impl PicCanisterTrait for CyclesLedgerPic {
    /// The shared PocketIc instance.
    fn pic(&self) -> Arc<PocketIc> {
        self.pic.clone()
    }
    /// The ID of this canister.
    fn canister_id(&self) -> Principal {
        self.canister_id.clone()
    }
}

impl From<PicCanister> for CyclesLedgerPic {
    fn from(pic: PicCanister) -> Self {
        Self {
            pic: pic.pic,
            canister_id: pic.canister_id,
        }
    }
}

impl CyclesLedgerPic {
    /*
    pub fn create_canister(&self, caller: Principal, arg0: &CreateCanisterArgs) -> Result<(std::result::Result<CreateCanisterSuccess, CreateCanisterError>,)> {
      self.pic.update_call(self.canister_id, caller, "create_canister", (arg0,))
    }
    pub fn create_canister_from(&self, caller: Principal, arg0: &CreateCanisterFromArgs) -> Result<(std::result::Result<CreateCanisterSuccess, CreateCanisterFromError>,)> {
      self.pic.update_call(self.canister_id, caller, "create_canister_from", (arg0,))
    }
    */
    pub fn deposit(&self, caller: Principal, arg0: &DepositArgs) -> Result<DepositResult, String> {
        self.update(caller, "deposit", (arg0,))
    }
    /*
    pub fn http_request(&self, caller: Principal, arg0: &HttpRequest) -> Result<(HttpResponse,)> {
      self.pic.update_call(self.canister_id, caller, "http_request", (arg0,))
    }
    */
    pub fn icrc_1_balance_of(
        &self,
        caller: Principal,
        arg0: &Account,
    ) -> Result<candid::Nat, String> {
        self.update(caller, "icrc1_balance_of", arg0)
    }
    /*
    pub fn icrc_1_decimals(&self, caller: Principal) -> Result<(u8,)> {
      self.pic.update_call(self.canister_id, caller, "icrc1_decimals", ())
    }
    pub fn icrc_1_fee(&self, caller: Principal) -> Result<(candid::Nat,)> {
      self.pic.update_call(self.canister_id, caller, "icrc1_fee", ())
    }
    pub fn icrc_1_metadata(&self, caller: Principal) -> Result<(Vec<(String,MetadataValue,)>,)> {
      self.pic.update_call(self.canister_id, caller, "icrc1_metadata", ())
    }
    pub fn icrc_1_minting_account(&self, caller: Principal) -> Result<(Option<Account>,)> {
      self.pic.update_call(self.canister_id, caller, "icrc1_minting_account", ())
    }
    pub fn icrc_1_name(&self, caller: Principal) -> Result<(String,)> {
      self.pic.update_call(self.canister_id, caller, "icrc1_name", ())
    }
    pub fn icrc_1_supported_standards(&self, caller: Principal) -> Result<(Vec<SupportedStandard>,)> {
      self.pic.update_call(self.canister_id, caller, "icrc1_supported_standards", ())
    }
    pub fn icrc_1_symbol(&self, caller: Principal) -> Result<(String,)> {
      self.pic.update_call(self.canister_id, caller, "icrc1_symbol", ())
    }
    pub fn icrc_1_total_supply(&self, caller: Principal) -> Result<(candid::Nat,)> {
      self.pic.update_call(self.canister_id, caller, "icrc1_total_supply", ())
    }
    pub fn icrc_1_transfer(&self, caller: Principal, arg0: &TransferArgs) -> Result<(std::result::Result<BlockIndex, TransferError>,)> {
      self.pic.update_call(self.canister_id, caller, "icrc1_transfer", (arg0,))
    }
    pub fn icrc_2_allowance(&self, caller: Principal, arg0: &AllowanceArgs) -> Result<(Allowance,)> {
      self.pic.update_call(self.canister_id, caller, "icrc2_allowance", (arg0,))
    }
    */
    pub fn icrc_2_approve(
        &self,
        caller: Principal,
        arg0: &ApproveArgs,
    ) -> std::result::Result<std::result::Result<candid::Nat, ApproveError>, String> {
        self.update(caller, "icrc2_approve", arg0)
    }
    /*
     pub fn icrc_2_transfer_from(&self, caller: Principal, arg0: &TransferFromArgs) -> Result<(std::result::Result<candid::Nat, TransferFromError>,)> {
       self.pic.update_call(self.canister_id, caller, "icrc2_transfer_from", (arg0,))
     }
     pub fn icrc_3_get_archives(&self, caller: Principal, arg0: &GetArchivesArgs) -> Result<(GetArchivesResult,)> {
       self.pic.update_call(self.canister_id, caller, "icrc3_get_archives", (arg0,))
     }
     pub fn icrc_3_get_blocks(&self, caller: Principal, arg0: &GetBlocksArgs) -> Result<(GetBlocksResult,)> {
       self.pic.update_call(self.canister_id, caller, "icrc3_get_blocks", (arg0,))
     }
     pub fn icrc_3_get_tip_certificate(&self, caller: Principal) -> Result<(Option<DataCertificate>,)> {
       self.pic.update_call(self.canister_id, caller, "icrc3_get_tip_certificate", ())
     }
     pub fn icrc_3_supported_block_types(&self, caller: Principal) -> Result<(Vec<SupportedBlockType>,)> {
       self.pic.update_call(self.canister_id, caller, "icrc3_supported_block_types", encode_args(()))
     }
     pub fn withdraw(&self, caller: Principal, arg0: &WithdrawArgs) -> Result<(std::result::Result<BlockIndex, WithdrawError>,)> {
       self.pic.update_call(self.canister_id, caller, "withdraw", encode_args((arg0,)).unwrap())
     }
    */
    pub fn withdraw_from(
        &self,
        caller: Principal,
        arg0: &WithdrawFromArgs,
    ) -> Result<(std::result::Result<BlockIndex, WithdrawFromError>,), String> {
        self.update(caller, "withdraw_from", arg0)
    }
}
