use std::env;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use anyhow::{Error, Result};
use log::error;
use poise::PrefixFrameworkOptions;
use reqwest::{Client as Reqwest, ClientBuilder as ReqwestBuilder};
use serenity::prelude::*;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::markov::Markov;

mod commands;
mod events;
mod markov;

#[derive(Debug)]
pub struct DataWrapper(Arc<Data>);
type Context<'a> = poise::Context<'a, DataWrapper, Error>;
type FrameworkContext<'a> = poise::FrameworkContext<'a, DataWrapper, Error>;

#[derive(Debug)]
pub struct Data {
    markov: Arc<Markov>,
    markov_loop_running: AtomicBool,
    reqwest: Reqwest,
    postgres: PgPool,
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
async fn main() -> Result<()> {
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
                    error!("on_error: {:?}", e);
                    if let Some(ctx) = e.ctx() {
                        if ctx.framework().options.commands.contains(ctx.command()) {
                            if let Err(e) = ctx
                                .say("An error occurred while executing this command.")
                                .await
                            {
                                error!("could not reply in on_error: {e}");
                            };
                        }
                    }
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
                    reqwest: ReqwestBuilder::new().pool_max_idle_per_host(1).build()?,
                    postgres: PgPoolOptions::new()
                        .connect(&env::var("DATABASE_URL")?)
                        .await?,
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

    Ok(())
}
