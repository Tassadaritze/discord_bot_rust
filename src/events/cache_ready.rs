use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;

use serenity::client::Context;
use serenity::futures::StreamExt;
use serenity::model::id::{ChannelId, GuildId};

use crate::markov::Markov;
use crate::Handler;

pub async fn handle(handler: &Handler, ctx: Context, _guilds: Vec<GuildId>) {
    let ctx = Arc::new(ctx);

    if !handler.is_loop_running.load(Ordering::Relaxed) {
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

        handler.is_loop_running.swap(true, Ordering::Relaxed);
    }
}
