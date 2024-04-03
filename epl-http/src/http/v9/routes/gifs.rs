use axum::{Extension, Json};
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_derive::{Deserialize, Serialize};
use epl_common::options::{EplOptions, Options};
use epl_common::tenor;
use epl_common::tenor::{get_gif_categories, get_suggested_search_terms, Gif, search_tenor};
use crate::AppState;
use crate::authorization_extractor::SessionContext;

#[derive(Serialize, Deserialize)]
pub struct SearchQueryParams {
    q: String,
    media_format: String,
    provider: String,
    locale: Option<String>
}

pub async fn search_gifs(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Query(params): Query<SearchQueryParams>
) -> impl IntoResponse {
    let options = EplOptions::get();

    if params.provider.eq("tenor") {
        if options.tenor_key.is_none() {
            return StatusCode::BAD_REQUEST.into_response()
        }

        let gifs = search_tenor(options.tenor_key.unwrap(), params.q, params.locale, params.media_format).await;

        Json(gifs).into_response()
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}

#[derive(Serialize, Deserialize)]
pub struct TrendingQueryParams {
    media_format: String,
    provider: String,
    locale: Option<String>
}

#[derive(Serialize, Deserialize)]
pub struct SimplifiedCategory {
    name: String,
    src: String
}

#[derive(Serialize)]
pub struct TrendingRes {
    categories: Vec<SimplifiedCategory>,
    gifs: Vec<Gif>
}

pub async fn get_trending_gifs(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Query(params): Query<TrendingQueryParams>
) -> impl IntoResponse {
    let options = EplOptions::get();

    if params.provider.eq("tenor") {
        if options.tenor_key.is_none() {
            return StatusCode::BAD_REQUEST.into_response()
        }

        let tenor_key = options.tenor_key.unwrap();

        let categories = get_gif_categories(tenor_key.clone(), params.locale.clone()).await;
        let trending = tenor::get_trending_gifs(tenor_key, 1, params.locale, params.media_format).await;

        Json(
            TrendingRes {
                categories: categories.iter().map(|x|
                    SimplifiedCategory {
                        name: x.searchterm.clone(),
                        // EVIL EVIL EVIL
                        src: x.image.clone().replace(".gif", ".mp4"),
                    }
                ).collect(),
                gifs: trending,
            }
        ).into_response()
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}

pub async fn actually_get_trending_gifs(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Query(params): Query<TrendingQueryParams>
) -> impl IntoResponse {
    let options = EplOptions::get();

    if params.provider.eq("tenor") {
        if options.tenor_key.is_none() {
            return StatusCode::BAD_REQUEST.into_response()
        }

        let gifs = tenor::get_trending_gifs(options.tenor_key.unwrap(), 50, params.locale, params.media_format).await;

        Json(gifs).into_response()
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}

#[derive(Serialize, Deserialize)]
pub struct SuggestionQueryParams {
    q: String,
    provider: String,
    limit: i32,
    locale: Option<String>
}

pub async fn gif_search_suggestions(
    Extension(state): Extension<AppState>,
    Extension(session_context): Extension<SessionContext>,
    Query(params): Query<SuggestionQueryParams>
) -> impl IntoResponse {
    let options = EplOptions::get();

    if params.provider.eq("tenor") {
        if options.tenor_key.is_none() {
            return StatusCode::BAD_REQUEST.into_response()
        }

        let suggestions = get_suggested_search_terms(options.tenor_key.unwrap(), params.q, params.limit, params.locale).await;

        Json(suggestions).into_response()
    } else {
        StatusCode::BAD_REQUEST.into_response()
    }
}