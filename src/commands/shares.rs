use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Error, Result};
use poise::CreateReply;
use serenity::all::Context as SerenityContext;
use serenity::all::{
    CacheHttp, Colour, ComponentInteraction, CreateActionRow, CreateButton, CreateEmbed,
    EditInteractionResponse, UserId,
};
use sqlx::{query, query_as, SqlitePool};

use crate::{Context, FrameworkContext};

pub const COLLECT_BUTTON: &str = "collect";
pub const BUY_GENERATOR_BUTTON: &str = "buy_generator";

#[derive(Debug)]
struct Shares {
    user_id: String,
    shares: i64,
    generators: i64,
    collection_time: Option<i64>,
    generation_time: i64,
}

impl Shares {
    /// Return `Shares` for user with `user_id`.
    async fn fetch_one(user_id: &str, sqlite: &SqlitePool) -> Result<Self> {
        Ok(
            query_as!(Self, "SELECT * FROM share WHERE user_id = ?", user_id)
                .fetch_one(sqlite)
                .await?,
        )
    }

    /// The base amount of time (in seconds) until a share can be collected again or a generator runs once.
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

    /// Tick generators if enough time has passed.
    async fn update(&mut self, sqlite: &SqlitePool) -> Result<()> {
        let ticks = ((unix_now()? - self.generation_time) / Self::COLLECTION_COOLDOWN).max(0);

        if ticks < 1 {
            return Ok(());
        } else {
            self.generation_time += Self::COLLECTION_COOLDOWN * ticks;
            self.shares += self.generators * ticks;

            query!(
                "UPDATE share
                SET (shares, generation_time) = (?, ?)
                WHERE user_id = ?",
                self.shares,
                self.generation_time,
                self.user_id
            )
            .execute(sqlite)
            .await?;
        }

        Ok(())
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

    let mut shares = match query_as!(Shares, "SELECT * FROM share WHERE user_id = ?", author_id)
        .fetch_optional(&sqlite)
        .await?
    // would LOVE to do this with .unwrap_or_else()
    {
        Some(shares) => shares,
        None => {
            let now: i64 = unix_now()?;
            query!(
                "INSERT INTO share(user_id, generation_time) VALUES(?, ?)",
                author_id,
                now
            )
            .execute(&sqlite)
            .await?;
            Shares::fetch_one(&author_id, &sqlite).await?
        }
    };

    shares.update(&sqlite).await?;

    ctx.send(
        CreateReply::new()
            .embed(
                CreateEmbed::new()
                    .colour(Colour::from_rgb(231, 41, 57))
                    .title(format!(
                        "You have {}🩸 shares! (+{}🩸/hr)",
                        shares.shares, shares.generators
                    ))
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
                CreateButton::new(COLLECT_BUTTON)
                    .label("Collect Shares")
                    .emoji('🩸')
                    .disabled(!shares.can_collect()?),
                CreateButton::new(BUY_GENERATOR_BUTTON)
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
    let sqlite = ctx.data().sqlite.clone();

    ctx.defer().await?;

    let mut shares_vec = query_as!(Shares, "SELECT * FROM share")
        .fetch_all(&sqlite)
        .await?;

    for shares in shares_vec.iter_mut() {
        shares.update(&sqlite).await?;
    }
    shares_vec.sort_unstable_by(|a, b| b.shares.partial_cmp(&a.shares).unwrap());

    let mut fields: Vec<(String, String, bool)> = vec![];
    for (i, shares) in shares_vec.iter().take(10).enumerate() {
        let user = UserId::new(shares.user_id.parse()?)
            .to_user(ctx.http())
            .await?;
        let user_name = user
            .nick_in(ctx.http(), ctx.guild_id().unwrap_or_default())
            .await
            .unwrap_or(user.name);
        fields.push((
            format!(
                "{}. {} | {}🩸 | {}🏭",
                i + 1,
                user_name,
                shares.shares,
                shares.generators
            ),
            String::new(),
            false,
        ));
    }

    ctx.send(
        CreateReply::new().embed(
            CreateEmbed::new()
                .colour(Colour::from_rgb(231, 41, 57))
                .title("Shares Leaderboard")
                .fields(fields),
        ),
    )
    .await?;

    Ok(())
}

pub async fn on_collect(
    framework_ctx: FrameworkContext<'_>,
    ctx: &SerenityContext,
    interaction: &ComponentInteraction,
) -> Result<()> {
    let sqlite = framework_ctx.user_data.sqlite.clone();

    interaction.defer_ephemeral(&ctx.http).await?;

    let mut shares = Shares::fetch_one(&interaction.user.id.to_string(), &sqlite).await?;
    shares.update(&sqlite).await?;

    if shares.can_collect()? {
        shares.collection_time = Some(unix_now()?);
        shares.shares += 1;
        query!(
            "UPDATE share
                SET (shares, collection_time) = (?, ?)
                WHERE user_id = ?",
            shares.shares,
            shares.collection_time,
            shares.user_id
        )
        .execute(&sqlite)
        .await?;
        interaction
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().content(format!(
                    "Shares collected! You now have {}🩸 shares.",
                    shares.shares
                )),
            )
            .await?;
    } else {
        interaction
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().content(format!(
                    "You cannot collect shares right now. \
                    You can collect shares <t:{}:R>.",
                    shares.collection_time.unwrap() + Shares::COLLECTION_COOLDOWN
                )),
            )
            .await?;
    }

    Ok(())
}

pub async fn on_buy_generator(
    framework_ctx: FrameworkContext<'_>,
    ctx: &SerenityContext,
    interaction: &ComponentInteraction,
) -> Result<()> {
    let sqlite = framework_ctx.user_data.sqlite.clone();

    interaction.defer_ephemeral(&ctx.http).await?;

    let mut shares = Shares::fetch_one(&interaction.user.id.to_string(), &sqlite).await?;
    shares.update(&sqlite).await?;

    let cost = shares
        .next_generator_cost()
        .ok_or_else(|| Error::msg("couldn't get generator cost"))?;
    if shares.shares >= cost {
        shares.shares -= cost;
        shares.generators += 1;
        query!(
            "UPDATE share
                SET (shares, generators) = (?, ?)
                WHERE user_id = ?",
            shares.shares,
            shares.generators,
            shares.user_id
        )
        .execute(&sqlite)
        .await?;
        interaction
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().content(format!(
                    "Generator purchased! You now have {}🩸 shares.",
                    shares.shares
                )),
            )
            .await?;
    } else {
        interaction
            .edit_response(
                &ctx.http,
                EditInteractionResponse::new().content(format!(
                    "You cannot afford another generator right now. \
                    You have {}🩸 shares and your next generator costs {}🩸.",
                    shares.shares, cost
                )),
            )
            .await?;
    }

    Ok(())
}

/// Get system time in seconds since the Unix epoch
fn unix_now() -> Result<i64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs()
        .try_into()?)
}
