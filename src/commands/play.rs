use super::*;

use crate::commands::join::join_channel;
use serenity::async_trait;
use serenity::{
    builder::{CreateEmbed, CreateEmbedFooter, CreateMessage},
    http::Http,
    model::{colour::Colour, id::GenericChannelId, user::User},
};
use songbird::{
    events::{Event, EventHandler},
    input::*,
    tracks::Track,
    EventContext, TrackEvent,
};
use std::sync::Arc;

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

#[poise::command(prefix_command, guild_only, aliases("p"))]
pub async fn play(ctx: Context<'_>, #[rest] msg: String) -> Result<(), Error> {
    let http_client = ctx.data().client.clone();
    let songbird = ctx.data().songbird.clone();

    let guild_id = ctx.guild_id().unwrap();
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

    let input = if msg.starts_with("http") {
        YoutubeDl::new(http_client, msg)
    } else {
        YoutubeDl::new_search(http_client, msg)
    };
    let mut input = Input::from(input);

    let metadata = input.aux_metadata().await.ok();

    let metadata = metadata.and_then(|metadata| {
        Some(Metadata::new(
            metadata.title.unwrap(),
            metadata.source_url.unwrap(),
            metadata.thumbnail.unwrap(),
        ))
    });

    let track = Track::from(input).pause();

    let event_handler = TrackHandler::new(ctx, metadata.clone()).await;
    let handler = call.lock().await.enqueue(track).await;

    // add handler that sends message when song starts playing
    handler
        .add_event(Event::Track(TrackEvent::Play), event_handler)
        .unwrap();

    // send queue message if there is already playing song
    let driver = call.lock().await;
    if driver.queue().len() > 1 && metadata.is_some() {
        ctx.channel_id()
            .send_message(
                &ctx.http(),
                CreateMessage::new().embed(EmbedInfo::create_embed(
                    metadata.unwrap(),
                    "Added to queue",
                    ctx.author().clone(),
                )),
            )
            .await?;
    }

    Ok(())
}
// struct for creating embed messages
struct EmbedInfo;

impl EmbedInfo {
    fn create_embed(metadata: Metadata, text: &str, author: User) -> CreateEmbed<'_> {
        CreateEmbed::new()
            .title(metadata.title)
            .url(metadata.url)
            .description(format!("**{text}**"))
            .thumbnail(metadata.thumbnail)
            .color(Colour::MAGENTA)
            .footer(EmbedInfo::create_footer(author))
    }
    fn create_footer(author: User) -> CreateEmbedFooter<'static> {
        CreateEmbedFooter::new(author.name.clone())
            .text(format!("Invoked by: {}", author.name))
            .icon_url(author.avatar_url().unwrap_or_default())
    }
}

// handler that sends message when the song starts playing
struct TrackHandler {
    metadata: Option<Metadata>,
    http: Arc<Http>,
    channel: GenericChannelId,
    author: User,
}

impl TrackHandler {
    async fn new(ctx: Context<'_>, metadata: Option<Metadata>) -> Self {
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
        if self.metadata.is_none() {
            return None;
        }

        let _ = self
            .channel
            .send_message(
                &self.http,
                CreateMessage::new().embed(EmbedInfo::create_embed(
                    self.metadata.clone().unwrap(),
                    "is playing",
                    self.author.clone(),
                )),
            )
            .await;

        None
    }
}
