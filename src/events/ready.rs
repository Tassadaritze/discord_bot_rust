use anyhow::Result;
use log::info;
use serenity::client::Context;
use serenity::model::gateway::Ready;

use crate::FrameworkContext;

pub async fn handle(
    _framework_ctx: FrameworkContext<'_>,
    _ctx: &Context,
    ready: &Ready,
) -> Result<()> {
    info!("{} online!", ready.user.name);

    Ok(())
}
