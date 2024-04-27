mod commands;

use commands::*;
use poise::{serenity_prelude as serenity, Framework, PrefixFrameworkOptions};
use reqwest::Client as HttpClient;
use serenity::prelude::{Client, TypeMapKey};
use songbird::SerenityInit;
struct HttpKey;

impl TypeMapKey for HttpKey {
    type Value = HttpClient;
}

#[tokio::main]
async fn main() {
    let framework = create_framework();

    let mut client = create_client(framework).await;

    client.start().await.unwrap();
}

fn create_framework() -> Framework<Data, Error> {
    poise::Framework::builder()
        .setup(|_, _, _| Box::pin(async move { Ok(()) }))
        .options(poise::FrameworkOptions {
            commands: vec![join(), play(), skip(), seek()],
            prefix_options: PrefixFrameworkOptions {
                prefix: Some(">".into()),
                case_insensitive_commands: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .build()
}

async fn create_client(framework: Framework<Data, Error>) -> Client {
    let token = std::env::var("DISCORD_TOKEN").expect("missing token");
    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .register_songbird()
        .type_map_insert::<HttpKey>(HttpClient::new())
        .await
        .expect("error creating client")
}
