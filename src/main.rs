use std::env;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::Error;
use log::error;
use poise::PrefixFrameworkOptions;
use serenity::prelude::*;

use crate::markov::Markov;

mod commands;
mod events;
mod markov;

pub struct DataWrapper(Arc<Data>);
type Context<'a> = poise::Context<'a, DataWrapper, Error>;
type FrameworkContext<'a> = poise::FrameworkContext<'a, DataWrapper, Error>;

pub struct Data {
    markov: Arc<Markov>,
    markov_loop_running: AtomicBool,
}

impl Deref for DataWrapper {
    type Target = Data;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TypeMapKey for Markov {
    type Value = Arc<Markov>;
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let token = env::var("DISCORD_TOKEN").expect("could not get discord token");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_SCHEDULED_EVENTS;

    let framework = poise::Framework::new(
        poise::FrameworkOptions {
            commands: commands::commands(),
            listener: |event, ctx, _| Box::pin(events::handle(ctx, event)),
            on_error: |e| {
                Box::pin(async move {
                    error!("on_error: {e}");
                })
            },
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("~".to_string()),
                mention_as_prefix: false,
                ..PrefixFrameworkOptions::default()
            },
            ..poise::FrameworkOptions::default()
        },
        |_, _, _| {
            Box::pin(async {
                Ok(DataWrapper(Arc::new(Data {
                    markov: Arc::new(Markov::new(2, "message-dump.txt", true)),
                    markov_loop_running: AtomicBool::new(false),
                })))
            })
        },
    );

    let mut client = Client::builder(token, intents)
        .framework(framework)
        .await
        .expect("error creating client");

    if let Err(why) = client.start().await {
        error!("client error: {:?}", why);
    }
}
