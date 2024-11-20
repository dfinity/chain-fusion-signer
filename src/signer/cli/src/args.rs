/// Manages Orbit on the Internet Computer.
use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(version, about)]
pub struct SignerCliArgs {
    /// Name of the station to execute the command on. (Uses default station if unspecified)
    #[clap(short, long, conflicts_with = "station_file")]
    pub network: Option<String>,
}
