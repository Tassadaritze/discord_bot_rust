use serenity::client::Context;
use serenity::model::channel::Message;

use crate::markov::Markov;
use crate::Handler;

pub async fn handle(_: &Handler, ctx: Context, new_message: Message) {
    match new_message.mentions_me(&ctx).await {
        Ok(does_mention) => {
            if does_mention {
                let mut reply: String = String::new();
                {
                    let data = ctx.data.read().await;
                    let markov = data.get::<Markov>();
                    match markov {
                        Some(markov) => reply = markov.generate_string().await,
                        None => eprintln!("could not get Markov from client data"),
                    }
                }
                if !reply.is_empty() {
                    if let Err(err) = new_message.reply_ping(&ctx, reply).await {
                        eprintln!("could not send reply to message {}: {err}", new_message.id);
                    }
                }
            }
        }
        Err(err) => eprintln!(
            "could not check if message {} mentions me: {err}",
            new_message.id
        ),
    }
}
