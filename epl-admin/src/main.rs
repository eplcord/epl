use std::env;
use clap::{Parser, Subcommand};
use tracing::log::debug;
use epl_common::rustflake;
use epl_common::tenor::ContentFormat::mp4;
use epl_common::tenor::{get_gif_categories, get_suggested_search_terms, get_trending_gifs, search_tenor};

#[derive(Clone)]
struct AdminOptions {
    tenor_key: Option<String>
}

pub trait Options {
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
    /// Test tenor's search
    TenorSearch {
        /// Term to search
        search_query: String
    },
    /// Test Tenor's categories
    TenorCategories,
    /// Test Tenor's trending gifs
    TenorTrending {
        /// Number of result wanted
        limit: i32
    },
    /// Test Tenor's search suggestions
    TenorSuggestions {
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
        Commands::TenorSearch { search_query } => {
            let tenor_key = options.tenor_key.unwrap();

            debug!("{:?}", search_tenor(tenor_key, search_query, None, String::from("mp4")).await);
        }
        Commands::TenorCategories => {
            let tenor_key = options.tenor_key.unwrap();

            debug!("{:?}", get_gif_categories(tenor_key, None).await);
        }
        Commands::TenorTrending { limit } => {
            let tenor_key = options.tenor_key.unwrap();

            debug!("{:?}", get_trending_gifs(tenor_key, limit, None, String::from("mp4")).await);
        }
        Commands::TenorSuggestions { search_query, limit } => {
            let tenor_key = options.tenor_key.unwrap();

            debug!("{:?}", get_suggested_search_terms(tenor_key, search_query, limit, None).await)
        }
    }
}
