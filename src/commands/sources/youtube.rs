use super::{Metadata, Result, SourceError as Error, TrackInfo, TrackSource};
use reqwest::header::HeaderMap;
use reqwest::Client as HttpClient;
use rusty_ytdl as ytdl;
use serde::Deserialize;
use serenity::async_trait;
use songbird::input::{HttpRequest, Input};
use std::vec::IntoIter;
use tokio::process::Command;
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

// get best quality thumbnail
pub fn choose_thumbnail(mut thumbnails: Vec<Thumbnail>) -> Thumbnail {
    let len = thumbnails.len();
    thumbnails.sort_by_key(|thumbnail| thumbnail.width * thumbnail.height);

    thumbnails.remove(len - 1)
}
pub struct YoutubeSource {
    tracks: IntoIter<String>,
    client: HttpClient,
}

#[async_trait]
impl TrackSource for YoutubeSource {
    async fn next(&mut self) -> Option<Result<TrackInfo>> {
        let url = self.tracks.next()?;

        // TODO: Better error handling
        let video = match ytdl::Video::new_with_options(url, VIDEO_OPTIONS.clone()) {
            Ok(video) => video,
            Err(err) => return Some(Err(Error::Other)),
        };

        let info = match video.get_info().await {
            Ok(info) => info,
            Err(err) => return Some(Err(Error::Other)),
        };

        let format = match ytdl::choose_format(&info.formats, &VIDEO_OPTIONS) {
            Ok(format) => format,
            Err(err) => return Some(Err(Error::Other)),
        };

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
            client: self.client.clone(),
            request: format.url,
            headers: HeaderMap::default(),
            content_length,
        };

        Some(Ok(TrackInfo::new(metadata, input.into())))
    }
}

impl YoutubeSource {
    pub async fn new(query: &str, client: HttpClient) -> Result<Self> {
        let tracks = if query.contains("&list=") {
            YoutubeSource::from_playlist(query).await?
        } else if query.starts_with("http") {
            vec![query.to_string()]
        } else {
            YoutubeSource::from_query(query).await?
        };

        Ok(YoutubeSource {
            client,
            tracks: tracks.into_iter(),
        })
    }
    async fn from_query(query: &str) -> Result<Vec<String>> {
        let yt = YouTube::new().map_err(|_err: VideoError| Error::Other)?;

        let Video(video) = yt
            .search_one(query, SEARCH_OPTIONS)
            .await
            .map_err(|_err| Error::Other)?
            .ok_or(Error::Other)?
        else {
            return Err(Error::Other);
        };

        Ok(vec![video.url])
    }
    async fn from_playlist(url: &str) -> Result<Vec<String>> {
        let args = ["-J", "-s", "--flat-playlist", url];

        let mut output = Command::new("yt-dlp")
            .args(args)
            .output()
            .await
            .map_err(|_err| Error::Other)?;
        let result = output.stdout;

        #[derive(Deserialize)]
        struct Root {
            entries: Vec<Option<Entry>>,
        }

        #[derive(Deserialize)]
        struct Entry {
            url: String,
        }

        let data = serde_json::from_slice::<Root>(&result).map_err(|_err| Error::Other)?;

        let tracks = data
            .entries
            .into_iter()
            .flatten()
            .map(|entry| entry.url)
            .collect::<Vec<_>>();

        Ok(tracks)
    }
}
