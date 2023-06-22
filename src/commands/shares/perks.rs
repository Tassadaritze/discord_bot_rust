use anyhow::Result;
use serenity::all::{
    ButtonStyle, ComponentInteraction, CreateEmbed, CreateEmbedFooter, EditInteractionResponse,
};
use serenity::all::{Context as SerenityContext, CreateActionRow, CreateButton};
use sqlx::query;

use super::Shares;
use crate::FrameworkContext;

pub struct Perk<'a> {
    name: &'a str,
    description: &'a str,
}

impl<'a> Perk<'a> {
    const fn new(name: &'a str, description: &'a str) -> Self {
        Self { name, description }
    }
}

pub trait FromName {
    fn electric_love(&self) -> usize;
    fn prism_cube(&self) -> usize;
    fn spiral(&self) -> usize;
    fn dance_robot_dance(&self) -> usize;
    fn world_is_mine(&self) -> usize;
}

impl FromName for [Perk<'_>] {
    fn electric_love(&self) -> usize {
        0
    }
    fn prism_cube(&self) -> usize {
        1
    }
    fn spiral(&self) -> usize {
        2
    }
    fn dance_robot_dance(&self) -> usize {
        3
    }
    fn world_is_mine(&self) -> usize {
        4
    }
}

pub const PERKS: [Perk; 5] = [
    Perk::new(
        "Electric Love",
        "Manual collection now gives (10 * 🏭generators) 🩸shares.",
    ),
    Perk::new(
        "Prism Cube",
        "🏭Generators are boosted by (log10 of 🩸shares).",
    ),
    Perk::new("Spiral", "🏭Generators boost 🏭generators by 10% each."),
    Perk::new("Dance Robot Dance", "🏭Generators are 4x stronger."),
    Perk::new(
        "World is Mine",
        "🏭Generators now cost (1.6 ^ 🏭generators) instead.",
    ),
];

pub async fn on_perk_shop(
    framework_ctx: FrameworkContext<'_>,
    ctx: &SerenityContext,
    interaction: &ComponentInteraction,
    perk_index: usize,
) -> Result<()> {
    let postgres = framework_ctx.user_data.postgres.clone();

    interaction.defer_ephemeral(&ctx.http).await?;

    let shares = Shares::fetch_one(interaction.user.id.get().try_into()?, &postgres).await?;

    interaction
        .edit_response(
            &ctx.http,
            EditInteractionResponse::new()
                .embed(
                    CreateEmbed::new()
                        .footer(CreateEmbedFooter::new(format!(
                            "You have {} prestige points.",
                            shares.prestige_points
                        )))
                        .title(if shares.perks[perk_index] {
                            PERKS[perk_index].name.to_string() + " (purchased)"
                        } else {
                            PERKS[perk_index].name.to_string()
                        })
                        .description(PERKS[perk_index].description),
                )
                .components(vec![CreateActionRow::Buttons(vec![
                    CreateButton::new(format!(
                        "perk_shop:{}",
                        (perk_index as isize - 1).rem_euclid(PERKS.len() as isize)
                    ))
                    .emoji('⬅'),
                    CreateButton::new(format!("buy_perk:{}", perk_index))
                        .emoji('🛒')
                        .style(ButtonStyle::Secondary)
                        .disabled(shares.perks[perk_index] || shares.prestige_points < 1),
                    CreateButton::new(format!("perk_shop:{}", (perk_index + 1) % PERKS.len()))
                        .emoji('➡'),
                ])]),
        )
        .await?;

    Ok(())
}

pub async fn on_buy_perk(
    framework_ctx: FrameworkContext<'_>,
    ctx: &SerenityContext,
    interaction: &ComponentInteraction,
    perk_index: usize,
) -> Result<()> {
    let postgres = framework_ctx.user_data.postgres.clone();

    interaction.defer_ephemeral(&ctx.http).await?;

    let mut shares = Shares::fetch_one(interaction.user.id.get().try_into()?, &postgres).await?;

    if shares.prestige_points > 0 {
        shares.prestige_points -= 1;
        shares.perks.set(perk_index, true);
        query!(
            "UPDATE share
                SET (prestige_points, perks) = ($1, $2)
                WHERE user_id = $3",
            shares.prestige_points,
            shares.perks,
            shares.user_id
        )
        .execute(&postgres)
        .await?;
        interaction
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().content(format!(
                    "Perk {} purchased! You now have {} prestige points.",
                    PERKS[perk_index].name, shares.prestige_points
                )),
            )
            .await?;
    } else {
        interaction
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().content(format!(
                    "You cannot afford to buy a perk right now. \
                    You have {} prestige points and perks cost 1.",
                    shares.prestige_points
                )),
            )
            .await?;
    }

    Ok(())
}
