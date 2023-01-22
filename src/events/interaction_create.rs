use serenity::client::Context;
use serenity::model::application::interaction::{Interaction, InteractionResponseType};

use macros::run_commands;

use crate::Handler;

pub async fn handle(_: &Handler, ctx: Context, interaction: Interaction) {
    if let Interaction::ApplicationCommand(command) = interaction {
        println!("Received command interaction: {:#?}", command);

        let content = run_commands!();

        if let Err(why) = command
            .create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message.content(content))
            })
            .await
        {
            println!("Cannot respond to slash command: {}", why);
        }
    }
}
