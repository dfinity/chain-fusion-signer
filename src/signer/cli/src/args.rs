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

    /// Increase verbosity level
    #[clap(short, long, action = clap::ArgAction::Count, conflicts_with = "quiet")]
    pub(crate) verbose: u8,

    /// Reduce verbosity level
    #[clap(short, long, action = clap::ArgAction::Count, conflicts_with = "verbose")]
    pub(crate) quiet: u8,
}
