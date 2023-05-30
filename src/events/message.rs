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
        let reply = framework_ctx.user_data.markov.generate_string().await;
        new_message.reply_ping(&ctx, reply).await?;
    }

    Ok(())
}
