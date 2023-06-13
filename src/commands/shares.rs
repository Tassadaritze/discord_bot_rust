use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use log::debug;
use poise::CreateReply;
use serenity::all::{Colour, CreateActionRow, CreateButton, CreateEmbed};
use sqlx::{query, query_as};

use crate::Context;

#[derive(Debug)]
struct Shares {
    user_id: String,
    shares: i64,
    generators: i64,
    collection_time: Option<i64>,
    last_update: i64,
}

impl Shares {
    /// How long (in seconds) until a share can be collected again.
    const COLLECTION_COOLDOWN: i64 = 60 * 60;

    /// Get the amount of shares it would take to make another generator.
    ///
    /// Returns `None` if the cost would overflow, or if amount of generators fails to convert to u32.
    fn next_generator_cost(&self) -> Option<i64> {
        2_i64.checked_pow(self.generators.try_into().ok()?)
    }

    /// Whether or not shares can be collected right now.
    fn can_collect(&self) -> Result<bool> {
        if let Some(collection_time) = self.collection_time {
            Ok(unix_now()? - collection_time > Self::COLLECTION_COOLDOWN)
        } else {
            Ok(true)
        }
    }

    /// Whether or not a new generator can be bought right now.
    fn can_buy_generator(&self) -> bool {
        if let Some(cost) = self.next_generator_cost() {
            self.shares >= cost
        } else {
            false
        }
    }
}

#[poise::command(slash_command, subcommands("get", "leaderboard"))]
pub async fn shares(_ctx: Context<'_>) -> Result<()> {
    Ok(())
}

/// View and collect shares
#[poise::command(slash_command)]
pub async fn get(ctx: Context<'_>) -> Result<()> {
    let author_id = ctx.author().id.to_string();
    let sqlite = ctx.data().sqlite.clone();

    let shares = match query_as!(Shares, "SELECT * FROM share WHERE user_id = ?", author_id)
        .fetch_optional(&sqlite)
        .await?
    // would LOVE to do this with .unwrap_or_else()
    {
        Some(shares) => shares,
        None => {
            let now: i64 = unix_now()?;
            query!(
                "INSERT INTO share(user_id, last_update) VALUES(?, ?)",
                author_id,
                now
            )
            .execute(&sqlite)
            .await?;
            query_as!(Shares, "SELECT * FROM share WHERE user_id = ?", author_id)
                .fetch_one(&sqlite)
                .await?
        }
    };

    debug!("{:?}", shares);

    ctx.send(
        CreateReply::new()
            .embed(
                CreateEmbed::new()
                    .colour(Colour::from_rgb(231, 41, 57))
                    .title(format!("You have {}🩸 shares!", shares.shares))
                    .field(
                        "Next 🩸Shares Collection",
                        format!(
                            "<t:{}:R>",
                            if let Some(time) = shares.collection_time {
                                time + Shares::COLLECTION_COOLDOWN
                            } else {
                                unix_now()?
                            }
                        ),
                        true,
                    )
                    .field("🏭Generators", shares.generators.to_string(), true)
                    .field(
                        "Next 🏭Generator Cost",
                        if let Some(cost) = shares.next_generator_cost() {
                            cost.to_string() + "🩸"
                        } else {
                            "INFINITY".to_string() + "🩸"
                        },
                        true,
                    ),
            )
            .components(vec![CreateActionRow::Buttons(vec![
                CreateButton::new("collect")
                    .label("Collect Shares")
                    .emoji('🩸')
                    .disabled(!shares.can_collect()?),
                CreateButton::new("buy_generator")
                    .label("Buy Generator")
                    .emoji('🏭')
                    .disabled(!shares.can_buy_generator()),
            ])]),
    )
    .await?;

    Ok(())
}

/// View users with the most shares
#[poise::command(slash_command)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<()> {
    ctx.say("Leaderboard goes here").await?;

    Ok(())
}

/// Gets system time in seconds since the Unix epoch
fn unix_now() -> Result<i64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs()
        .try_into()?)
}
