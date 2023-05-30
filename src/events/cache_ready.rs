use std::sync::atomic::Ordering;
use std::time::Duration;

use anyhow::Result;
use log::error;
use serenity::all::CreateMessage;
use serenity::client::Context;
use serenity::futures::StreamExt;
use serenity::model::id::{ChannelId, GuildId};

use crate::FrameworkContext;

pub async fn handle(
    framework_ctx: FrameworkContext<'_>,
    ctx: &Context,
    _guilds: &[GuildId],
) -> Result<()> {
    let data = framework_ctx.user_data;
    let cache = ctx.cache.clone();
    let http = ctx.http.clone();
    let markov = data.markov.clone();

    if !data.markov_loop_running.load(Ordering::Relaxed) {
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
                {
                    const MARKOV_CHANNEL: ChannelId = ChannelId::new(464502359372857355);
                    let http = http.clone();

                    let last_message = MARKOV_CHANNEL.messages_iter(&http).boxed().next().await;
                    if let Some(result) = last_message {
                        match result {
                            Ok(message) => {
                                if message.author.id != cache.current_user().id {
                                    let generated_message = markov.generate_string().await;
                                    MARKOV_CHANNEL
                                        .send_message(
                                            &http,
                                            CreateMessage::new().content(generated_message),
                                        )
                                        .await
                                        .expect("couldn't send message");
                                }
                            }
                            Err(err) => error!("error getting message: {err}"),
                        }
                    }
                }
            }
        });

        data.markov_loop_running.store(true, Ordering::Relaxed);
    }

    Ok(())
}
