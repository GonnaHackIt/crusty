use reqwest::header::HeaderMap;
use reqwest::Client as HttpClient;
use rusty_ytdl as ytdl;
use songbird::input::{HttpRequest, Input};
use ytdl::{
    search::{SearchOptions, SearchResult::*, SearchType, YouTube},
    DownloadOptions, RequestOptions, Thumbnail, VideoError, VideoOptions, VideoQuality,
    VideoSearchOptions,
};

static SEARCH_OPTIONS: Option<&SearchOptions> = Some(&SearchOptions {
    limit: 1,
    search_type: SearchType::Video,
    safe_search: false,
});

static VIDEO_OPTIONS: VideoOptions = VideoOptions {
    filter: VideoSearchOptions::Audio,
    quality: VideoQuality::HighestAudio,
    download_options: DownloadOptions {
        dl_chunk_size: None,
    },
    request_options: RequestOptions {
        proxy: None,
        cookies: None,
        ipv6_block: None,
    },
};

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

// Searches for a song with a given query
pub async fn search(query: &str, client: HttpClient) -> Result<(Metadata, Input), VideoError> {
    let yt = YouTube::new()?;

    let Video(video) = yt
        .search_one(query, SEARCH_OPTIONS)
        .await?
        .ok_or(VideoError::VideoNotFound)?
    else {
        return Err(VideoError::VideoNotFound);
    };

    play_url(&video.url, client).await
}

// Gets the video from the given url and prepares input for songbird
pub async fn play_url(url: &str, client: HttpClient) -> Result<(Metadata, Input), VideoError> {
    let video = ytdl::Video::new_with_options(url, VIDEO_OPTIONS.clone())?;
    let info = video.get_info().await?;

    let format = ytdl::choose_format(&info.formats, &VIDEO_OPTIONS)?;

    let data = info.video_details;
    let metadata = Metadata::new(
        data.title,
        data.video_url,
        choose_thumbnail(data.thumbnails).url,
    );

    let content_length: Option<u64> = match format.content_length {
        Some(len) => str::parse::<u64>(&len).ok(),
        None => None,
    };

    let input = HttpRequest {
        client,
        request: format.url,
        headers: HeaderMap::default(),
        content_length,
    };

    Ok((metadata, input.into()))
}

// get best quality thumbnail
pub fn choose_thumbnail(mut thumbnails: Vec<Thumbnail>) -> Thumbnail {
    let len = thumbnails.len();
    thumbnails.sort_by_key(|thumbnail| thumbnail.width * thumbnail.height);

    thumbnails.remove(len - 1)
}
