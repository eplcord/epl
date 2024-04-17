use clap::Subcommand;
use epl_common::tenor::{get_gif_categories, get_suggested_search_terms, get_trending_gifs, search_tenor};
use crate::AdminOptions;

#[derive(Debug, Subcommand)]
pub(crate) enum TenorCommands {
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

pub(crate) async fn tenor_commands(options: AdminOptions, tenor: TenorCommands) {
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