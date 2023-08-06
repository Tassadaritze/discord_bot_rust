use anyhow::Result;
use serenity::all::{GuildId, UserId, VoiceState};
use serenity::client::Context;

use crate::FrameworkContext;

const RP_GUILD: GuildId = GuildId::new(464502358924197912);
const ME: UserId = UserId::new(392352456303968256);

pub async fn handle(
    _framework_ctx: FrameworkContext<'_>,
    ctx: &Context,
    _old: &Option<VoiceState>,
    new: &VoiceState,
) -> Result<()> {
    if new.guild_id.is_some_and(|id| id == RP_GUILD) {
        if let Some(channel_id) = new.channel_id {
            let channel = ctx.http.get_channel(channel_id).await?;
            if channel.guild().is_some_and(|channel| {
                channel.members(ctx.cache.clone()).is_ok_and(|members| {
                    members.len() >= 4 && !members.iter().any(|member| member.user.id == ME)
                })
            }) {
                ME.create_dm_channel(&ctx.http)
                    .await?
                    .say(&ctx.http, "IT'S TIME")
                    .await?;
            }
        }
    }

    Ok(())
}
