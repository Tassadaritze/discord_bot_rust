use anyhow::Result;
use serenity::all::{Context, Interaction};

use crate::commands::shares::{on_buy_generator, on_collect, BUY_GENERATOR_BUTTON, COLLECT_BUTTON};
use crate::FrameworkContext;

pub async fn handle(
    framework_ctx: FrameworkContext<'_>,
    ctx: &Context,
    interaction: &Interaction,
) -> Result<()> {
    if let Interaction::Component(interaction) = interaction {
        match interaction.data.custom_id.as_str() {
            COLLECT_BUTTON => {
                on_collect(framework_ctx, ctx, interaction).await?;
            }
            BUY_GENERATOR_BUTTON => {
                on_buy_generator(framework_ctx, ctx, interaction).await?;
            }
            _ => (),
        };
    };

    Ok(())
}
