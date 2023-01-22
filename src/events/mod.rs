use serenity::model::id::ChannelId;

use macros::get_modules;

get_modules!("src/events");

const EVENT_REPORT_CHANNEL: ChannelId = ChannelId(924343631761006592);
