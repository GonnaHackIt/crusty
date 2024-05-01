pub mod youtube;

use reqwest::Client as HttpClient;
use serenity::async_trait;
use songbird::input::Input;
pub use youtube::*;

type Result<T> = std::result::Result<T, SourceError>;

pub struct SourceFactory;

impl SourceFactory {
    pub async fn new(query: &str, client: HttpClient) -> Result<Box<dyn TrackSource>> {
        // Default source is Youtube
        let source = YoutubeSource::new(query, client).await?;

        Ok(Box::new(source))
    }
}

// TODO: More Errors
pub enum SourceError {
    Other,
}

#[async_trait]
// trait for getting next tracks from the source
pub trait TrackSource: Send {
    async fn next(&mut self) -> Option<Result<TrackInfo>>;
}

pub struct TrackInfo {
    pub metadata: Metadata,
    pub input: Input,
}

impl TrackInfo {
    fn new(metadata: Metadata, input: Input) -> Self {
        TrackInfo { metadata, input }
    }
}

#[derive(Clone)]
pub struct Metadata {
    pub title: String,
    pub url: String,
    pub thumbnail: String,
}

impl Metadata {
    fn new(title: String, url: String, thumbnail: String) -> Self {
        Metadata {
            title,
            url,
            thumbnail,
        }
    }
}
