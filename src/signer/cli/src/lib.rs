//! Off-chain client for the chain fusion signer.

use args::SignerCliArgs;
use candid::Principal;
use dfx_core::{
    config::model::canister_id_store::CanisterIdStore,
    interface::{builder::IdentityPicker, dfx::DfxInterface},
};
use logger::init_logger;
use slog::Logger;
pub mod args;
pub mod logger;
use anyhow::Context;
use ic_cdk::api::management_canister::ecdsa::{
    EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument, EcdsaPublicKeyResponse,
};
use ic_chain_fusion_signer_api::types::generic::GenericCallerEcdsaPublicKeyError;

pub struct SignerCli {
    dfx_interface: DfxInterface,
    /// A logger; some public `sdk` repository methods require a specific type of logger so this is a compatible logger.
    logger: Logger,
}

impl SignerCli {
    pub async fn execute(args: SignerCliArgs) -> anyhow::Result<()> {
        let signer_cli = Self::new(args).await?;
        let public_key = signer_cli.ecdsa_public_key().await?;
        println!("{:?}", public_key);
        Ok(())
    }
    pub async fn new(config: SignerCliArgs) -> anyhow::Result<Self> {
        let SignerCliArgs {
            network,
            identity,
            verbose,
            quiet,
        } = config;

        let dfx_interface = Self::dfx_interface(network, identity).await?;
        let logger = init_logger(verbose, quiet)?;

        Ok(Self {
            dfx_interface,
            logger,
        })
    }

    /// Gets the dfx_core interface
    pub(crate) async fn dfx_interface(
        network_name: Option<String>,
        identity: Option<String>,
    ) -> anyhow::Result<DfxInterface> {
        let network_name = network_name.unwrap_or_else(|| "local".to_string());
        let identity = identity
            .map(IdentityPicker::Named)
            .unwrap_or(IdentityPicker::Selected);
        let interface_builder = DfxInterface::builder()
            .with_identity(identity)
            .with_network_named(&network_name);
        let interface = interface_builder.build().await?;
        if !interface.network_descriptor().is_ic {
            interface.agent().fetch_root_key().await?;
        }
        Ok(interface)
    }

    /// Gets the ID of a given canister name.  If the name is already an ID, it is returned as is.
    pub fn canister_id(&self, canister_name: &str) -> anyhow::Result<Principal> {
        let canister_id_store = CanisterIdStore::new(
            &self.logger,
            self.dfx_interface.network_descriptor(),
            self.dfx_interface.config(),
        )?;

        let canister_id = Principal::from_text(canister_name).or_else(|_| {
            canister_id_store.get(canister_name).with_context(|| {
                format!(
                    "Failed to look up principal id for canister named \"{}\"",
                    canister_name
                )
            })
        })?;

        Ok(canister_id)
    }

    pub async fn ecdsa_public_key(&self) -> anyhow::Result<EcdsaPublicKeyResponse> {
        let signer_canister_id = self
            .canister_id("signer")
            .expect("Signer canister ID is not known");
        let key_name = "dfx_test_key".to_string(); // TODO: "key_1" on mainnet
        let response_bytes = self
            .dfx_interface
            .agent()
            .update(&signer_canister_id, "generic_caller_ecdsa_public_key")
            .with_arg(
                candid::encode_one(EcdsaPublicKeyArgument {
                    canister_id: None,
                    derivation_path: vec![],
                    key_id: EcdsaKeyId {
                        curve: EcdsaCurve::Secp256k1,
                        name: key_name,
                    },
                })
                .with_context(|| "Failed to encode argument")?,
            )
            .call_and_wait()
            .await
            .with_context(|| "Failed to make canister call")?;
        let response = candid::decode_one::<
            Result<(EcdsaPublicKeyResponse,), GenericCallerEcdsaPublicKeyError>,
        >(&response_bytes)
        .with_context(|| "Failed to decode response")?;
        let response = match response {
            Ok((response,)) => response,
            Err(err) => panic!("Failed to get pubkey: {:?}", err),
        };

        Ok(response)
    }
}
