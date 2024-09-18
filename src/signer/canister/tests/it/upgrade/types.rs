use candid::{CandidType, Deserialize, Principal};
use ic_chain_fusion_signer_api::types::token::ChainId;
use ic_chain_fusion_signer_api::types::Version;

#[derive(CandidType, Deserialize)]
pub struct InitArgV0_0_25 {
    pub ecdsa_key_name: String,
    pub allowed_callers: Vec<Principal>,
}

#[derive(CandidType, Deserialize)]
pub enum ArgV0_0_25 {
    Init(InitArgV0_0_25),
    Upgrade,
}
