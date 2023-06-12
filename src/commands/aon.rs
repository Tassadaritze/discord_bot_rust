use std::env;

use anyhow::Result;
use serde::Deserialize;

use crate::Context;

/// Search Archives of Nethys
#[poise::command(
    slash_command,
    description_localized("ru", "Поиск по Archives of Nethys")
)]
pub async fn aon(
    ctx: Context<'_>,
    #[description = "What to search"]
    #[description_localized("ru", "Что искать")]
    #[name_localized("ru", "запрос")]
    query: String,
) -> Result<()> {
    let reqwest = ctx.framework().user_data.reqwest.clone();

    ctx.defer().await?;

    let res = reqwest
        .get(env::var("SEARCH_URL")?)
        .query(&[
            ("format", "json"),
            (
                "q",
                &("site:aonprd.com -site:2e.aonprd.com ".to_string() + &query),
            ),
        ])
        .send()
        .await?
        .json::<Response>()
        .await?;

    let response = match res.results.get(0) {
        Some(result) => &result.url,
        None => "No results.",
    };
    ctx.say(response).await?;

    Ok(())
}

#[derive(Deserialize, Debug)]
struct SearchResult {
    url: String,
}

#[derive(Deserialize, Debug)]
struct Response {
    results: Vec<SearchResult>,
}
