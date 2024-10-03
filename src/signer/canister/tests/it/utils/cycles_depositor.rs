#![allow(dead_code, unused_imports)]
use std::sync::Arc;

use candid::{self, CandidType, Deserialize, Principal};
use ic_cdk::api::call::CallResult as Result;
use pocket_ic::PocketIc;

use super::pic_canister::{PicCanister, PicCanisterTrait};

#[derive(CandidType, Deserialize, Debug)]
pub(crate) struct InitArg {
    pub(crate) ledger_id: Principal,
}
#[derive(CandidType, Deserialize, Debug)]
pub(crate) struct Account {
    pub(crate) owner: Principal,
    pub(crate) subaccount: Option<serde_bytes::ByteBuf>,
}
#[derive(CandidType, Deserialize, Debug)]
pub(crate) struct DepositArg {
    pub(crate) to: Account,
    pub(crate) memo: Option<serde_bytes::ByteBuf>,
    pub(crate) cycles: candid::Nat,
}
#[derive(CandidType, Deserialize, Debug)]
pub(crate) struct DepositResult {
    pub(crate) balance: candid::Nat,
    pub(crate) block_index: candid::Nat,
}

pub struct Service(pub Principal);
impl Service {
    pub async fn deposit(&self, arg0: &DepositArg) -> Result<(DepositResult,)> {
        ic_cdk::call(self.0, "deposit", (arg0,)).await
    }
}

pub struct CyclesDepositorPic {
    pub pic: Arc<PocketIc>,
    pub canister_id: Principal,
}

impl From<PicCanister> for CyclesDepositorPic {
    fn from(pic: PicCanister) -> Self {
        Self {
            pic: pic.pic(),
            canister_id: pic.canister_id(),
        }
    }
}

impl PicCanisterTrait for CyclesDepositorPic {
    /// The shared PocketIc instance.
    fn pic(&self) -> Arc<PocketIc> {
        self.pic.clone()
    }
    /// The ID of this canister.
    fn canister_id(&self) -> Principal {
        self.canister_id.clone()
    }
}

impl CyclesDepositorPic {
    pub fn deposit(
        &self,
        _caller: Principal,
        arg0: &DepositArg,
    ) -> std::result::Result<DepositResult, String> {
        self.update(self.canister_id, "deposit", arg0)
    }
}
