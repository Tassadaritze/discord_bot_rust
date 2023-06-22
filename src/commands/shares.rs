use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use poise::CreateReply;
use serenity::all::Context as SerenityContext;
use serenity::all::{
    CacheHttp, Colour, ComponentInteraction, CreateActionRow, CreateButton, CreateEmbed,
    EditInteractionResponse, UserId,
};
use sqlx::{query, query_as, PgPool};

use crate::{Context, FrameworkContext};

pub const COLLECT_BUTTON: &str = "collect";
pub const BUY_GENERATOR_BUTTON: &str = "buy_generator";

#[derive(Debug)]
struct Shares {
    user_id: i64,
    shares: f32,
    generators: i32,
    collection_time: Option<DateTime<Utc>>,
    generation_time: DateTime<Utc>,
}

impl Shares {
    /// Return `Shares` for user with `user_id`.
    async fn fetch_one(user_id: i64, postgres: &PgPool) -> Result<Self> {
        Ok(
            query_as!(Self, "SELECT * FROM share WHERE user_id = $1", user_id)
                .fetch_one(postgres)
                .await?,
        )
    }

    /// The base amount of time (in seconds) until a share can be collected again or a generator runs once.
    const COLLECTION_COOLDOWN: i32 = 60 * 60;

    /// Get the amount of shares it would take to make another generator.
    fn next_generator_cost(&self) -> f32 {
        2_f32.powi(self.generators)
    }

    /// Whether or not shares can be collected right now.
    fn can_collect(&self) -> Result<bool> {
        if let Some(collection_time) = self.collection_time {
            Ok(Utc::now() - collection_time > Duration::seconds(Self::COLLECTION_COOLDOWN as i64))
        } else {
            Ok(true)
        }
    }

    /// Whether or not a new generator can be bought right now.
    fn can_buy_generator(&self) -> bool {
        self.shares >= self.next_generator_cost()
    }

    /// Tick generators if enough time has passed.
    async fn update(&mut self, postgres: &PgPool) -> Result<()> {
        let ticks =
            ((Utc::now() - self.generation_time) / Self::COLLECTION_COOLDOWN).num_seconds() as i32;

        if ticks < 1 {
            return Ok(());
        } else {
            self.generation_time += Duration::seconds((Self::COLLECTION_COOLDOWN * ticks) as i64);
            self.shares += (self.generators * ticks) as f32;

            query!(
                "UPDATE share
                SET (shares, generation_time) = ($1, $2)
                WHERE user_id = $3",
                self.shares,
                self.generation_time,
                self.user_id
            )
            .execute(postgres)
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
    let author_id: i64 = ctx.author().id.get().try_into()?;
    let postgres = ctx.data().postgres.clone();

    let mut shares = match query_as!(Shares, "SELECT * FROM share WHERE user_id = $1", author_id)
        .fetch_optional(&postgres)
        .await?
    // would LOVE to do this with .unwrap_or_else()
    {
        Some(shares) => shares,
        None => {
            query!(
                "INSERT INTO share(user_id) VALUES($1)",
                author_id,
            )
            .execute(&postgres)
            .await?;
            Shares::fetch_one(author_id, &postgres).await?
        }
    };

    shares.update(&postgres).await?;

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
                                (time + Duration::seconds(Shares::COLLECTION_COOLDOWN as i64))
                                    .timestamp()
                            } else {
                                Utc::now().timestamp()
                            }
                        ),
                        true,
                    )
                    .field("🏭Generators", shares.generators.to_string(), true)
                    .field(
                        "Next 🏭Generator Cost",
                        shares.next_generator_cost().to_string(),
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
    let postgres = ctx.data().postgres.clone();

    ctx.defer().await?;

    let mut shares_vec = query_as!(Shares, "SELECT * FROM share")
        .fetch_all(&postgres)
        .await?;

    for shares in shares_vec.iter_mut() {
        shares.update(&postgres).await?;
    }
    shares_vec.sort_unstable_by(|a, b| b.shares.partial_cmp(&a.shares).unwrap());

    let mut fields: Vec<(String, String, bool)> = Vec::new();
    for (i, shares) in shares_vec.iter().take(10).enumerate() {
        let user = UserId::new(shares.user_id.try_into()?)
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
    let postgres = framework_ctx.user_data.postgres.clone();

    interaction.defer_ephemeral(&ctx.http).await?;

    let mut shares = Shares::fetch_one(interaction.user.id.get().try_into()?, &postgres).await?;
    shares.update(&postgres).await?;

    if shares.can_collect()? {
        shares.collection_time = Some(Utc::now());
        shares.shares += 1.0;
        query!(
            "UPDATE share
                SET (shares, collection_time) = ($1, $2)
                WHERE user_id = $3",
            shares.shares,
            shares.collection_time,
            shares.user_id
        )
        .execute(&postgres)
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
                    (shares.collection_time.unwrap()
                        + Duration::seconds(Shares::COLLECTION_COOLDOWN as i64))
                    .timestamp()
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
    let postgres = framework_ctx.user_data.postgres.clone();

    interaction.defer_ephemeral(&ctx.http).await?;

    let mut shares = Shares::fetch_one(interaction.user.id.get().try_into()?, &postgres).await?;
    shares.update(&postgres).await?;

    let cost = shares.next_generator_cost();
    if shares.shares >= cost {
        shares.shares -= cost;
        shares.generators += 1;
        query!(
            "UPDATE share
                SET (shares, generators) = ($1, $2)
                WHERE user_id = $3",
            shares.shares,
            shares.generators,
            shares.user_id
        )
        .execute(&postgres)
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
