use std::env;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use serenity::client::{Context, EventHandler};
use serenity::model::application::interaction::Interaction;
use serenity::model::gateway::Ready;
use serenity::model::guild::ScheduledEvent;
use serenity::model::id::GuildId;
use serenity::prelude::{GatewayIntents, TypeMapKey};
use serenity::{async_trait, Client};
use serenity::model::channel::Message;

use crate::markov::Markov;

mod commands;
mod events;
mod markov;

impl TypeMapKey for Markov {
    type Value = Arc<Markov>;
}

pub struct Handler {
    is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        events::cache_ready::handle(self, ctx, guilds).await;
    }

    async fn message(&self, ctx: Context, new_message: Message) {
        events::message::handle(self, ctx, new_message).await;
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        events::ready::handle(self, ctx, ready).await;
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        events::interaction_create::handle(self, ctx, interaction).await;
    }

    async fn guild_scheduled_event_create(&self, ctx: Context, event: ScheduledEvent) {
        events::guild_scheduled_event_create::handle(self, ctx, event).await;
    }

    async fn guild_scheduled_event_update(&self, ctx: Context, event: ScheduledEvent) {
        events::guild_scheduled_event_update::handle(self, ctx, event).await;
    }

    async fn guild_scheduled_event_delete(&self, ctx: Context, event: ScheduledEvent) {
        events::guild_scheduled_event_delete::handle(self, ctx, event).await;
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("could not get discord token");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_SCHEDULED_EVENTS;

    let mut client = Client::builder(token, intents)
        .event_handler(Handler {
            is_loop_running: AtomicBool::new(false),
        })
        .await
        .expect("error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Markov>(Arc::new(Markov::new(2, "message-dump.txt", true)));
    }

    if let Err(why) = client.start().await {
        println!("client error: {:?}", why);
    }
}
