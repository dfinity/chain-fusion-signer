//! Off-chain client for the chain fusion signer.

use anyhow::Context;
use args::SignerCliArgs;
use dfx_core::interface::{builder::IdentityPicker, dfx::DfxInterface};
use slog::Logger;
pub mod args;

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
    pub async fn new(
        network: Option<String>,
        identity: Option<String>,
        logger: Logger,
    ) -> anyhow::Result<Self> {
        let network = network.unwrap_or_else(|| "local".to_string());
        let dfx_interface = Self::dfx_interface(&network, identity).await?;

        Ok(Self {
            dfx_interface,
            logger,
        })
    }

    /// Gets the dfx_core interface
    pub(crate) async fn dfx_interface(
        network_name: &str,
        identity: Option<String>,
    ) -> anyhow::Result<DfxInterface> {
        let identity = identity
            .map(IdentityPicker::Named)
            .unwrap_or(IdentityPicker::Selected);
        let interface_builder = DfxInterface::builder()
            .with_identity(identity)
            .with_network_named(network_name);
        let interface = interface_builder.build().await?;
        if !interface.network_descriptor().is_ic {
            interface.agent().fetch_root_key().await?;
        }
        Ok(interface)
    }
}
