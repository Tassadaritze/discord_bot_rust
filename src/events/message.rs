use anyhow::Result;
use serenity::client::Context;
use serenity::model::channel::Message;

use crate::FrameworkContext;

pub async fn handle(
    framework_ctx: FrameworkContext<'_>,
    ctx: &Context,
    new_message: &Message,
) -> Result<()> {
    if new_message.mentions_me(&ctx).await? {
        loop {
            let reply = framework_ctx.user_data.markov.generate_string().await;
            if reply.len() < 2000 {
                new_message.reply_ping(&ctx, reply).await?;
                break;
            }
        }
    }

    Ok(())
}
