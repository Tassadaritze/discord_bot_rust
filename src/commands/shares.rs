use anyhow::Result;
use log::debug;
use sqlx::{query, Row};

use crate::Context;

#[poise::command(slash_command, subcommands("get", "leaderboard"))]
pub async fn shares(_ctx: Context<'_>) -> Result<()> {
    Ok(())
}

/// View and collect shares
#[poise::command(slash_command)]
pub async fn get(ctx: Context<'_>) -> Result<()> {
    let sqlite = ctx.data().sqlite.clone();

    let row = query("SELECT * FROM share WHERE user_id = 1234")
        .fetch_optional(&sqlite)
        .await?;

    if let Some(row) = row {
        debug!("{:?}", row.try_get::<&str, &str>("user_id")?);
    }

    ctx.say("Share view goes here").await?;

    Ok(())
}

/// View users with the most shares
#[poise::command(slash_command)]
pub async fn leaderboard(ctx: Context<'_>) -> Result<()> {
    ctx.say("Leaderboard goes here").await?;

    Ok(())
}
