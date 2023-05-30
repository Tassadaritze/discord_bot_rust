use anyhow::Result;
use log::error;
use serenity::all::CreateMessage;
use serenity::client::Context;
use serenity::model::guild::{ScheduledEvent, ScheduledEventStatus};

use super::EVENT_REPORT_CHANNEL;
use crate::FrameworkContext;

pub async fn handle(
    _framework_ctx: FrameworkContext<'_>,
    ctx: &Context,
    event: &ScheduledEvent,
) -> Result<()> {
    let report_channel_guild_id = match ctx.cache.guild_channel(EVENT_REPORT_CHANNEL) {
        Some(val) => val.guild_id,
        None => {
            error!("could not get EVENT_REPORT_CHANNEL as guild channel");
            return Ok(());
        }
    };
    if event.guild_id != report_channel_guild_id {
        return Ok(());
    }

    match event.status {
        ScheduledEventStatus::Scheduled => {
            EVENT_REPORT_CHANNEL
                .send_message(
                    &ctx,
                    CreateMessage::new().content(
                        String::from("Event **")
                            + &event.name
                            + "** updated (scheduled for <t:"
                            + &event.start_time.unix_timestamp().to_string()
                            + ">).",
                    ),
                )
                .await?;
        }
        ScheduledEventStatus::Active => {
            EVENT_REPORT_CHANNEL
                .send_message(
                    &ctx,
                    CreateMessage::new().content(
                        String::from("Event **")
                            + &event.name
                            + "** has started! <@&816024905061367829>",
                    ),
                )
                .await?;
        }
        _ => (),
    }

    Ok(())
}
