use anyhow::Result;
use log::info;
use serenity::all::Command;
use serenity::client::Context;
use serenity::model::gateway::Ready;

use crate::FrameworkContext;

pub async fn handle(
    framework_ctx: FrameworkContext<'_>,
    ctx: &Context,
    ready: &Ready,
) -> Result<()> {
    let create_commands =
        poise::builtins::create_application_commands(&framework_ctx.options.commands);
    Command::set_global_commands(&ctx.http, create_commands).await?;

    info!("{} online!", ready.user.name);

    Ok(())
}
