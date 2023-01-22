use serenity::client::Context;
use serenity::model::guild::ScheduledEvent;

use super::EVENT_REPORT_CHANNEL;
use crate::Handler;

pub async fn handle(_: &Handler, ctx: Context, event: ScheduledEvent) {
    let report_channel_guild_id = match &ctx.cache.guild_channel(EVENT_REPORT_CHANNEL) {
        Some(val) => val.guild_id,
        None => {
            eprintln!("could not get EVENT_REPORT_CHANNEL as guild channel");
            return;
        }
    };
    if event.guild_id != report_channel_guild_id {
        return;
    }

    if let Err(err) = EVENT_REPORT_CHANNEL
        .send_message(&ctx, |message| {
            message.content(
                String::from("Event **")
                    + &event.name
                    + "** was **cancelled** (previously scheduled for <t:"
                    + &event.start_time.unix_timestamp().to_string()
                    + ">).",
            )
        })
        .await
    {
        eprintln!("error sending scheduled event deletion report message: {err}");
    }
}
