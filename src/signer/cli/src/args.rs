/// Manages Orbit on the Internet Computer.
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct SignerCliArgs {
    /// Name of the dfx network
    #[clap(short, long)]
    pub network: Option<String>,
    /// Name of the identity to use
    #[clap(short, long)]
    pub identity: Option<String>,
}
