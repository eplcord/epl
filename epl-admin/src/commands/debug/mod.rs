use clap::Subcommand;
use unic::emoji::char::is_emoji;
use crate::AdminOptions;
use crate::commands::debug::tenor::{tenor_commands, TenorCommands};

mod tenor;

#[derive(Debug, Subcommand)]
pub(crate) enum DebugCommands {
    /// Test various Tenor related features
    #[command(subcommand)]
    Tenor(TenorCommands),
    /// Check to see if some text is a valid emoji
    Emoji {
        /// Text to check if it's a valid emoji
        text: char
    }
}

pub(crate) async fn debug_commands(options: AdminOptions, debug: DebugCommands) {
    match debug {
        DebugCommands::Tenor(tenor) => tenor_commands(options, tenor).await,
        DebugCommands::Emoji { text } => {
            println!("{}: {:?}", text, is_emoji(text))
        }
    }
}