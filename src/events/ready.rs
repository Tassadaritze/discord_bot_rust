use serenity::client::Context;
use serenity::model::application::command::Command;
use serenity::model::gateway::Ready;

use macros::register_commands;

use crate::Handler;

pub async fn handle(_: &Handler, ctx: Context, ready: Ready) {
    register_commands!();

    Command::set_global_application_commands(&ctx.http, register_commands)
        .await
        .expect("error setting global commands");

    println!("{} online!", ready.user.name);
}
