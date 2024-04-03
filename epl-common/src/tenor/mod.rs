use std::collections::HashMap;
use serde_derive::{Deserialize, Serialize};
use tracing::error;
use crate::tenor::ContentFormat::{gif, gifpreview, mp4, tinygif};

#[derive(Deserialize, Debug)]
pub struct Media {
    /// A URL to the media source
    url: String,
    /// Width and height of the media in pixels
    dims: Vec<i32>,
    /// Represents the time in seconds for one loop of the content. If the content is static,
    /// the duration is set to 0.
    duration: f32,
    /// Size of the file in bytes
    size: i32
}

#[derive(Deserialize, Debug)]
pub struct Category {
    /// The search term that corresponds to the category. The search term is translated to match
    /// the locale of the corresponding request.
    pub searchterm: String,
    /// The search URL to request if the user selects the category
    path: String,
    /// A URL to the media source for the category's example GIF
    pub image: String,
    /// Category name to overlay over the image. The name is translated to match the locale of the
    /// corresponding request.
    name: String,
}

#[derive(Deserialize, Debug)]
pub struct Response {
    /// A Unix timestamp that represents when this post was created.
    created: f32,
    /// Returns true if this post contains audio.
    hasaudio: bool,
    /// Tenor result identifier
    id: String,
    /// A dictionary with a content format as the key and a Media Object as the value.
    media_formats: HashMap<ContentFormat, Media>,
    /// An array of tags for the post
    tags: Vec<String>,
    /// The title of the post
    title: String,
    /// A textual description of the content.
    //
    // We recommend that you use content_description for user accessibility features.
    content_description: String,
    /// The full URL to view the post on tenor.com.
    itemurl: String,
    /// Returns true if this post contains captions.
    hascaption: Option<bool>,
    /// Comma-separated list to signify whether the content is a sticker or static image, has audio,
    /// or is any combination of these. If sticker and static aren't present, then the content is a
    /// GIF. A blank flags field signifies a GIF without audio.
    flags: Vec<String>,
    /// The most common background pixel color of the content
    bg_color: Option<String>,
    /// A short URL to view the post on tenor.com.
    url: String
}

#[derive(Deserialize, Eq, PartialEq, Hash, Debug)]
pub enum ContentFormat {
    preview,
    gif,
    mediumgif,
    tinygif,
    nanogif,
    mp4,
    loopedmp4,
    tinymp4,
    nanomp4,
    webm,
    tinywebm,
    nanowebm,
    webp_transparent,
    tinywebp_transparent,
    nanowebp_transparent,
    gif_transparent,
    tinygif_transparent,
    nanogif_transparent,
    tinywebppreview_transparent,
    gifpreview,
    nanowebppreview_transparent,
    webppreview_transparent,
    nanogifpreview,
    tinygifpreview,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Gif {
    id: String,
    title: String,
    url: String,
    src: String,
    gif_src: String,
    width: i32,
    height: i32,
    preview: String
}

#[derive(Deserialize)]
pub struct SearchResponse {
    results: Vec<Response>,
    next: String
}

pub async fn search_tenor(
    api_key: String,
    term: String,
    locale: Option<String>,
    format: String
) -> Vec<Gif> {
    let mut vec = vec![];

    let res = ureq::get(
        &format!(
            "https://tenor.googleapis.com/v2/search?q={}&key={}&limit=100&media_filter={},gif,gifpreview{}",
            term,
            api_key,
            format,
            if locale.is_some() {
                format!("&locale={}", locale.unwrap())
            } else {
                String::new()
            }
        )
    )
        .call()
        .expect("Failed to call Tenor API!")
        .into_json::<SearchResponse>();

    match res {
        Ok(res) => {
            for i in res.results {
                let gif_preview_media = i.media_formats.get(&gifpreview).unwrap();

                let wanted_format = match format.as_str() {
                    "mp4" => mp4,
                    "tinygif" => tinygif,
                    _ => gif
                };
                
                vec.push(
                  Gif {
                      id: i.id,
                      title: i.title,
                      url: i.itemurl,
                      src: i.media_formats.get(&wanted_format).unwrap().url.clone(),
                      gif_src: i.media_formats.get(&gif).unwrap().url.clone(),
                      width: gif_preview_media.dims[0],
                      height: gif_preview_media.dims[1],
                      preview: gif_preview_media.url.clone(),
                  }
                );
            }
        }
        Err(e) => {
            error!("{e}");
        }
    }

    vec
}

#[derive(Deserialize, Debug)]
pub struct CategoriesResponse {
    tags: Vec<Category>,
}

pub async fn get_gif_categories(
    api_key: String,
    locale: Option<String>
) -> Vec<Category> {
    let mut vec = vec![];

    let res = ureq::get(
        &format!(
            "https://tenor.googleapis.com/v2/categories?key={}&media_filter=mp4{}",
            api_key,
            if locale.is_some() {
                format!("&locale={}", locale.unwrap())
            } else {
                String::new()
            }
        )
    )
        .call()
        .expect("Failed to call Tenor API!")
        .into_json::<CategoriesResponse>();

    match res {
        Ok(res) => {
            vec = res.tags
        }
        Err(e) => {
            error!("{e}");
        }
    }

    vec
}

pub async fn get_trending_gifs(
    api_key: String,
    limit: i32,
    locale: Option<String>,
    format: String
) -> Vec<Gif> {
    let mut vec = vec![];

    let res = ureq::get(
        &format!(
            "https://tenor.googleapis.com/v2/featured?key={}&limit={}&media_filter={},gif,gifpreview{}",
            api_key,
            limit,
            format,
            if locale.is_some() {
                format!("&locale={}", locale.unwrap())
            } else {
                String::new()
            }
        )
    )
        .call()
        .expect("Failed to call Tenor API!")
        .into_json::<SearchResponse>();

    match res {
        Ok(res) => {
            for i in res.results {
                let gif_preview_media = i.media_formats.get(&gifpreview).unwrap();

                let wanted_format = match format.as_str() {
                    "mp4" => mp4,
                    "tinygif" => tinygif,
                    _ => gif
                };
                
                vec.push(
                    Gif {
                        id: i.id,
                        title: i.title,
                        url: i.itemurl,
                        src: i.media_formats.get(&wanted_format).unwrap().url.clone(),
                        gif_src: i.media_formats.get(&gif).unwrap().url.clone(),
                        width: gif_preview_media.dims[0],
                        height: gif_preview_media.dims[1],
                        preview: gif_preview_media.url.clone(),
                    }
                );
            }
        }
        Err(e) => {
            error!("{e}");
        }
    }

    vec
}

#[derive(Deserialize, Debug)]
pub struct SuggestionsResponse {
    locale: String,
    results: Vec<String>,
}

pub async fn get_suggested_search_terms(
    api_key: String,
    term: String,
    limit: i32,
    locale: Option<String>
) -> Vec<String> {
    let mut vec = vec![];

    let res = ureq::get(
        &format!(
            "https://tenor.googleapis.com/v2/search_suggestions?key={}&q={}&limit={}{}",
            api_key,
            term,
            limit,
            if locale.is_some() {
                format!("&locale={}", locale.unwrap())
            } else {
                String::new()
            }
        )
    )
        .call()
        .expect("Failed to call Tenor API!")
        .into_json::<SuggestionsResponse>();

    match res {
        Ok(res) => {
            vec = res.results
        }
        Err(e) => {
            error!("{e}");
        }
    }

    vec
}