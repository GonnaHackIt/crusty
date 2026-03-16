mod commands;
use commands::*;

use poise::{Framework, PrefixFrameworkOptions};
use serenity::{all::OnlineStatus, prelude::*};
use songbird::Songbird;
use std::sync::Arc;

const PREFIX: &str = ">";

struct UserData {
    pub songbird: Arc<Songbird>,
    pub client: reqwest::Client,
}

#[tokio::main]
async fn main() {
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Couldn't install crypto provider");

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let framework = create_framework();
    let mut client = create_client(framework).await;

    client.start().await.unwrap();
}

fn create_framework() -> Framework<Data, Error> {
    poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![join(), play(), skip(), seek()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(PREFIX.into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .build()
}

async fn create_client(framework: Framework<Data, Error>) -> Client {
    let token = Token::from_env("DISCORD_TOKEN").expect("Missing Discord Token!");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    let songbird = Songbird::serenity();
    let data = UserData {
        songbird: Arc::clone(&songbird),
        client: reqwest::Client::new(),
    };

    Client::builder(token, intents)
        .framework(Box::new(framework))
        .voice_manager(songbird)
        .data(Arc::new(data))
        .status(OnlineStatus::Idle)
        .await
        .expect("error creating client")
}
