use anyhow::Result;
use serenity::all::{Context, Interaction};

use crate::commands::shares::perks::{on_buy_perk, on_perk_shop};
use crate::commands::shares::{
    on_buy_generator, on_collect, on_prestige, on_prestige_confirm, BUY_GENERATOR_BUTTON,
    COLLECT_BUTTON, PRESTIGE_BUTTON, PRESTIGE_CONFIRM_BUTTON,
};
use crate::FrameworkContext;

pub async fn handle(
    framework_ctx: FrameworkContext<'_>,
    ctx: &Context,
    interaction: &Interaction,
) -> Result<()> {
    if let Interaction::Component(interaction) = interaction {
        let id = interaction.data.custom_id.as_str();
        if id.starts_with("perk_shop") {
            on_perk_shop(
                framework_ctx,
                ctx,
                interaction,
                id.split(':').last().unwrap().parse()?,
            )
            .await?;
        } else if id.starts_with("buy_perk") {
            on_buy_perk(
                framework_ctx,
                ctx,
                interaction,
                id.split(':').last().unwrap().parse()?,
            )
            .await?;
        } else {
            match id {
                COLLECT_BUTTON => {
                    on_collect(framework_ctx, ctx, interaction).await?;
                }
                BUY_GENERATOR_BUTTON => {
                    on_buy_generator(framework_ctx, ctx, interaction).await?;
                }
                PRESTIGE_BUTTON => {
                    on_prestige(framework_ctx, ctx, interaction).await?;
                }
                PRESTIGE_CONFIRM_BUTTON => {
                    on_prestige_confirm(framework_ctx, ctx, interaction).await?;
                }
                _ => (),
            };
        }
    };

    Ok(())
}
