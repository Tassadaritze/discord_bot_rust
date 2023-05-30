use anyhow::Result;

use crate::Context;

/// Replies with pong
#[poise::command(slash_command)]
pub async fn ping(ctx: Context<'_>) -> Result<()> {
    ctx.say("pong!").await?;

    Ok(())
}
