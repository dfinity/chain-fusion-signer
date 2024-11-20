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

pub struct SignerCli {
    dfx_interface: DfxInterface,
    /// A logger; some public `sdk` repository methods require a specific type of logger so this is a compatible logger.
    logger: Logger,
}

impl SignerCli {
    pub async fn execute(args: SignerCliArgs) -> anyhow::Result<String> {
        let signer_cli = Self::new(args).await?;
        let public_key = signer_cli.public_key().await?;
        Ok(public_key)
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

    pub async fn public_key(&self) -> anyhow::Result<String> {
        let signer_canister_id = self
            .canister_id("signer")
            .expect("Signer canister ID is not known");
        let x = self
            .dfx_interface
            .agent()
            .update(&signer_canister_id, "public_key");

        Ok("FIN".to_owned())
    }
}
