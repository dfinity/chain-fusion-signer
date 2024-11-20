//! Off-chain client for the chain fusion signer.

use anyhow::Context;
use args::SignerCliArgs;
use dfx_core::interface::{builder::IdentityPicker, dfx::DfxInterface};
use logger::init_logger;
use slog::Logger;
use candid::Principal;
pub mod args;
pub mod logger;

pub async fn execute(args: &SignerCliArgs) -> Result<String, String> {
    println!("Hello, world!");
    Ok("FIN".to_owned())
}

pub struct SignerCli {
    dfx_interface: DfxInterface,
    /// A logger; some public `sdk` repository methods require a specific type of logger so this is a compatible logger.
    logger: Logger,
}

impl SignerCli {
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

    pub async fn signer_canister_id(&self) -> Principal {
        unimplemented!()
    }

    pub async fn public_key(&self) -> anyhow::Result<String> {
        let signer_canister_id = self.signer_canister_id().await;
        let x = self.dfx_interface.agent().update(&signer_canister_id, "public_key");

        Ok("FIN".to_owned())
    }

}
