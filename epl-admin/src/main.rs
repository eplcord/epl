use clap::{Parser, Subcommand};
use tracing::log::debug;
use epl_common::options::{EplOptions, Options};
use epl_common::rustflake;

/// The Epl Administration CLI and Litecord Admin API HTTP Server
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Enable the Admin API HTTP Server
    Server
}

fn main() {
    tracing_subscriber::fmt::init();
    
    let args = Cli::parse();
    // let options = EplOptions::get();
    // let mut snowflake_factory = rustflake::Snowflake::default();
}
