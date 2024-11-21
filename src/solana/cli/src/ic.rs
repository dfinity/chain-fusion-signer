use candid::{CandidType, Decode, Deserialize, Encode};
use ic_agent::{export::Principal, identity::BasicIdentity, Agent, AgentError};
use serde_bytes::ByteBuf;

pub async fn create_agent(url: &str, is_mainnet: bool) -> Result<Agent, AgentError> {
    const DFX_REPLICA_PORT: &str="4943";
    const DFX_REPLICA_ADDRESS: &str="http://127.0.0.1:4943";

    let agent = Agent::builder()
    .with_url("http://localhost:4943")
        .with_identity(caller_id())
        .build()?;
    if !is_mainnet {
        agent.fetch_root_key().await?;
    }
    Ok(agent)
}

pub fn caller_id() -> BasicIdentity {
    BasicIdentity::from_pem_file(concat![
        env!("HOME"),
        "/.config/dfx/identity/default/identity.pem"
    ])
    .unwrap()
}

#[derive(CandidType, Deserialize, Debug, Eq, PartialEq, Clone)]
pub(crate) enum EcdsaCurve {
    #[serde(rename = "secp256k1")]
    Secp256K1,
}
#[derive(CandidType, Deserialize, Debug, Eq, PartialEq, Clone)]
pub(crate) struct EcdsaKeyId {
    pub(crate) name: String,
    pub(crate) curve: EcdsaCurve,
}
#[derive(CandidType, Deserialize, Debug, Eq, PartialEq, Clone)]
pub(crate) struct EcdsaPublicKeyArgument {
    pub(crate) key_id: EcdsaKeyId,
    pub(crate) canister_id: Option<Principal>,
    pub(crate) derivation_path: Vec<serde_bytes::ByteBuf>,
}
#[derive(CandidType, Deserialize, Debug, Eq, PartialEq, Clone)]
pub(crate) struct EcdsaPublicKeyResponse {
    pub(crate) public_key: serde_bytes::ByteBuf,
    pub(crate) chain_code: serde_bytes::ByteBuf,
}

pub async fn get_public_key(agent: &Agent) -> ByteBuf {
    let signer = Principal::from_text("grghe-syaaa-aaaar-qabyq-cai").unwrap();
    let response = agent
        .update(&signer, "generic_caller_ecdsa_public_key")
        .with_arg(
            Encode!(&EcdsaPublicKeyArgument {
                key_id: EcdsaKeyId {
                    curve: EcdsaCurve::Secp256K1,
                    name: "key_1".to_owned()
                },
                canister_id: None,
                derivation_path: vec![ByteBuf::from(String::from("/sol")), ByteBuf::default()]
            })
            .unwrap(),
        )
        .call_and_wait()
        .await
        .unwrap();
    let response = Decode!(&response, EcdsaPublicKeyResponse).expect("Failed to parse response");
    response.public_key
}
