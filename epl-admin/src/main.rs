use std::env;
use clap::{Args, Parser, Subcommand};
use unic::emoji::char::is_emoji;
use epl_common::tenor::{get_gif_categories, get_suggested_search_terms, get_trending_gifs, search_tenor};

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
enum Commands {
    /// Enable the Admin API HTTP Server
    Server,
    /// Various commands to do debugging
    #[command(subcommand)]
    Debug(DebugCommands)
}

#[derive(Debug, Subcommand)]
enum DebugCommands {
    /// Test various Tenor related features
    #[command(subcommand)]
    Tenor(TenorCommands),
    /// Check to see if some text is a valid emoji
    Emoji {
        /// Text to check if it's a valid emoji
        text: char
    }
}

#[derive(Debug, Subcommand)]
enum TenorCommands {
    /// Test tenor's search
    Search {
        /// Term to search
        search_query: String
    },
    /// Test Tenor's categories
    Categories,
    /// Test Tenor's trending gifs
    Trending {
        /// Number of result wanted
        limit: i32
    },
    /// Test Tenor's search suggestions
    Suggestions {
        /// Term to search
        search_query: String,
        /// Number of result wanted
        limit: i32
    }
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
        Commands::Debug(debug) => {
            match debug {
                DebugCommands::Tenor(tenor) => {
                    let tenor_key = options.tenor_key.expect("Please specify the TENOR_KEY environment variable!");
                    
                    match tenor {
                        TenorCommands::Search { search_query } => {
                            println!("{:?}", search_tenor(tenor_key, search_query, None, String::from("mp4")).await);
                        }
                        TenorCommands::Categories => {
                            println!("{:?}", get_gif_categories(tenor_key, None).await);
                        }
                        TenorCommands::Trending { limit } => {
                            println!("{:?}", get_trending_gifs(tenor_key, limit, None, String::from("mp4")).await);
                        }
                        TenorCommands::Suggestions { search_query, limit } => {
                            println!("{:?}", get_suggested_search_terms(tenor_key, search_query, limit, None).await)
                        }
                    }
                }
                DebugCommands::Emoji { text } => {
                    println!("{}: {:?}", text, is_emoji(text))
                }
            }
        }
    }
}
