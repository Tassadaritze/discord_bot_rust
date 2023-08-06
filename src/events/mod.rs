use anyhow::Result;
use serenity::all::{ChannelId, FullEvent};

use crate::FrameworkContext;

mod cache_ready;
mod guild_scheduled_event_create;
mod guild_scheduled_event_delete;
mod guild_scheduled_event_update;
mod interaction_create;
mod message;
mod ready;
mod voice_state_update;

const EVENT_REPORT_CHANNEL: ChannelId = ChannelId::new(924343631761006592);

pub async fn handle(framework_ctx: FrameworkContext<'_>, event: &FullEvent) -> Result<()> {
    match event {
        FullEvent::Ready {
            ctx,
            data_about_bot,
        } => ready::handle(framework_ctx, ctx, data_about_bot).await,
        FullEvent::CacheReady { ctx, guilds } => {
            cache_ready::handle(framework_ctx, ctx, guilds).await
        }
        FullEvent::Message { ctx, new_message } => {
            message::handle(framework_ctx, ctx, new_message).await
        }
        FullEvent::GuildScheduledEventCreate { ctx, event } => {
            guild_scheduled_event_create::handle(framework_ctx, ctx, event).await
        }
        FullEvent::GuildScheduledEventDelete { ctx, event } => {
            guild_scheduled_event_delete::handle(framework_ctx, ctx, event).await
        }
        FullEvent::GuildScheduledEventUpdate { ctx, event } => {
            guild_scheduled_event_update::handle(framework_ctx, ctx, event).await
        }
        FullEvent::InteractionCreate { ctx, interaction } => {
            interaction_create::handle(framework_ctx, ctx, interaction).await
        }
        FullEvent::VoiceStateUpdate { ctx, old, new } => {
            voice_state_update::handle(framework_ctx, ctx, old, new).await
        }
        _ => Ok(()),
    }
}
