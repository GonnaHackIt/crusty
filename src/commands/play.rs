use super::*;
use crate::commands::join::join_channel;
use crate::{HttpClient, HttpKey};
use poise::serenity_prelude as serenity;
use serenity::async_trait;
use serenity::builder::{CreateEmbed, CreateEmbedFooter, CreateMessage};
use songbird::{
    events::{Event, EventHandler},
    serenity::get as get_songbird,
    tracks::Track,
    EventContext, TrackEvent,
};
use sources::{Metadata, SourceFactory, TrackInfo};
use std::sync::Arc;

#[poise::command(prefix_command, guild_only, aliases("p"))]
pub async fn play(ctx: Context<'_>, #[rest] msg: String) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let songbird = get_songbird(ctx.serenity_context())
        .await
        .expect("Songbird not registered");

    // get voice connection, if doesnt exist, join channel and try again
    let call = match songbird.get(guild_id) {
        Some(call) => call,
        None => {
            join_channel(ctx).await?;
            let Some(call) = songbird.get(guild_id) else {
                return Ok(());
            };
            call
        }
    };

    ctx.say(format!("Searching: **`{msg}`**")).await?;

    let http_client = get_http_client(ctx).await;

    let Some(mut tracks_source) = SourceFactory::new(&msg, http_client).await else {
        ctx.say("Error during fetching source").await?;

        return Ok(());
    };

    while let Some(track) = tracks_source.next().await {
        let Ok(TrackInfo { metadata, input }) = track else {
            ctx.say("Error during fetching one of songs").await?;
            continue;
        };

        // pausing track so when added to queue it sends start playing event
        let track = Track::from(input).pause();

        let event_handler = TrackHandler::new(ctx, metadata.clone()).await;
        let handler = call.lock().await.enqueue(track).await;

        // add handler that sends message when song starts playing
        handler
            .add_event(Event::Track(TrackEvent::Play), event_handler)
            .unwrap();

        // send queue message if there is already playing song
        let driver = call.lock().await;
        if driver.queue().len() > 1 {
            ctx.channel_id()
                .send_message(
                    ctx.http(),
                    CreateMessage::new().embed(EmbedInfo::create_embed(
                        metadata,
                        "Added to queue",
                        ctx.author().clone(),
                    )),
                )
                .await?;
        }
    }

    Ok(())
}

async fn get_http_client(ctx: Context<'_>) -> HttpClient {
    // reqwest Client for rusty_ytdl
    ctx.serenity_context()
        .data
        .read()
        .await
        .get::<HttpKey>()
        .expect("Not registered Http Client")
        .clone()
}

// struct for creating embed messages
struct EmbedInfo;

impl EmbedInfo {
    fn create_embed(metadata: Metadata, text: &str, author: serenity::User) -> CreateEmbed {
        CreateEmbed::new()
            .title(metadata.title)
            .url(metadata.url)
            .description(format!("**{text}**"))
            .thumbnail(metadata.thumbnail)
            .color(serenity::Colour::MAGENTA)
            .footer(EmbedInfo::create_footer(author))
    }
    fn create_footer(author: serenity::User) -> CreateEmbedFooter {
        CreateEmbedFooter::new(author.name.clone())
            .text(format!("Invoked by: {}", author.name))
            .icon_url(author.avatar_url().unwrap_or_default())
    }
}

// handler that sends message when the song starts playing
struct TrackHandler {
    metadata: Metadata,
    http: Arc<serenity::Http>,
    channel: serenity::ChannelId,
    author: serenity::User,
}

impl TrackHandler {
    async fn new(ctx: Context<'_>, metadata: Metadata) -> Self {
        TrackHandler {
            metadata,
            http: Arc::clone(&ctx.serenity_context().http),
            channel: ctx.channel_id(),
            author: ctx.author().clone(),
        }
    }
}

#[async_trait]
impl EventHandler for TrackHandler {
    async fn act(&self, _ctx: &EventContext<'_>) -> Option<Event> {
        let _ = self
            .channel
            .send_message(
                self.http.clone(),
                CreateMessage::new().embed(EmbedInfo::create_embed(
                    self.metadata.clone(),
                    "is playing",
                    self.author.clone(),
                )),
            )
            .await;

        None
    }
}
