use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serenity::client::{Context, EventHandler};
use serenity::futures::StreamExt;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};
use serenity::model::gateway::Ready;
use serenity::model::id::{ChannelId, GuildId};
use serenity::prelude::{GatewayIntents, TypeMapKey};
use serenity::{async_trait, Client};

use crate::markov::Markov;
use commands_macro::{register_commands, run_commands};

mod commands;
mod markov;

impl TypeMapKey for Markov {
    type Value = Arc<Markov>;
}

struct Handler {
    is_loop_running: AtomicBool,
}

#[async_trait]
impl EventHandler for Handler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        let ctx = Arc::new(ctx);

        if !self.is_loop_running.load(Ordering::Relaxed) {
            let ctx = Arc::clone(&ctx);
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(3600)).await;
                    {
                        let ctx = Arc::clone(&ctx);
                        let last_message = ChannelId(464502359372857355)
                            .messages_iter(&ctx.http)
                            .boxed()
                            .next()
                            .await;
                        if let Some(result) = last_message {
                            match result {
                                Ok(message) => {
                                    if message.author.id != ctx.cache.current_user().id {
                                        let data = ctx.data.read().await;
                                        let markov = data
                                            .get::<Markov>()
                                            .expect("couldn't get Markov from client data");
                                        let generated_message = markov.generate_string().await;
                                        ChannelId(464502359372857355)
                                            .send_message(&ctx.http, |message| {
                                                message.content(generated_message)
                                            })
                                            .await
                                            .expect("couldn't send message");
                                    }
                                }
                                Err(err) => eprintln!("error getting message: {err}"),
                            }
                        }
                    }
                }
            });

            self.is_loop_running.swap(true, Ordering::Relaxed);
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        register_commands!();
        Command::set_global_application_commands(&ctx.http, register_commands)
            .await
            .expect("error setting global commands");

        println!("{} online!", ready.user.name);
    }

    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            println!("Received command interaction: {:#?}", command);

            let content = run_commands!();

            if let Err(why) = command
                .create_interaction_response(&ctx.http, |response| {
                    response
                        .kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|message| message.content(content))
                })
                .await
            {
                println!("Cannot respond to slash command: {}", why);
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("could not get discord token");

    let intents =
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILDS;

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
