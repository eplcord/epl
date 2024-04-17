mod commands;

use std::env;
use clap::{Parser, Subcommand};
use crate::commands::debug::{debug_commands, DebugCommands};
use crate::commands::users::{users_commands, UsersCommands};

#[derive(Clone)]
struct AdminOptions {
    tenor_key: Option<String>
}

trait Options {
    fn get() -> AdminOptions;
}

impl Options for AdminOptions {
    fn get() -> AdminOptions {
        AdminOptions {
            tenor_key: env::var("TENOR_KEY").ok()
        }
    }
}

/// The Epl Administration CLI and Litecord Admin API HTTP Server
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Enable the Admin API HTTP Server
    Server,
    /// Various commands to do debugging
    #[command(subcommand)]
    Debug(DebugCommands),
    #[command(subcommand)]
    Users(UsersCommands),
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let args = Cli::parse();
    let options = AdminOptions::get();
    // let mut snowflake_factory = rustflake::Snowflake::default();

    match args.command {
        Commands::Server => {
            unimplemented!()
        }
        Commands::Debug(debug) => debug_commands(options, debug).await,
        Commands::Users(users) => users_commands(options, users).await,
    }
}
