use std::collections::HashMap;
use std::env;

use anyhow::{Error, Result};
use poise::CreateReply;
use serde::Deserialize;
use serenity::all::CreateAttachment;
use serenity::http::CacheHttp;

use crate::Context;

/// Get an image from Gelbooru
#[poise::command(
    slash_command,
    description_localized("ru", "Получить картинку с Gelbooru")
)]
pub async fn gelbooru(
    ctx: Context<'_>,
    #[description = "Tags to search, in Gelbooru format"]
    #[description_localized("ru", "Искомые теги, в формате Gelbooru")]
    #[name_localized("ru", "теги")]
    tags: Option<String>,
) -> Result<()> {
    let reqwest = ctx.framework().user_data.reqwest.clone();
    let api = PostsAPI::new(tags.clone())?;

    ctx.defer().await?;

    let posts = reqwest
        .get(api.url())
        .send()
        .await?
        .json::<Response>()
        .await?;

    let post = posts
        .post
        .get(0)
        .ok_or_else(|| Error::msg("couldn't get post"))?;

    ctx.send(
        CreateReply::new()
            .content(format!("**{}:**", tags.unwrap_or("random".to_string())))
            .attachment(CreateAttachment::url(ctx.http(), &post.file_url).await?),
    )
    .await?;

    Ok(())
}

#[derive(Deserialize, Debug)]
struct Post {
    file_url: String,
}

#[derive(Deserialize, Debug)]
struct Response {
    post: Vec<Post>,
}

const GELBOORU_API_POSTS: &str = "https://gelbooru.com/index.php";

struct PostsAPI<'a> {
    pre_query: &'a str,
    query: HashMap<&'a str, String>,
}

impl<'a> PostsAPI<'a> {
    fn new(tags: Option<String>) -> Result<Self, env::VarError> {
        // todo: SFW/NSFW switch
        let tags = match tags {
            Some(tags) => tags.trim().to_string() + " sort:random rating:general",
            None => "sort:random rating:general".to_string(),
        };

        Ok(Self {
            pre_query: GELBOORU_API_POSTS,
            query: HashMap::from([
                ("page", "dapi".to_string()),
                ("s", "post".to_string()),
                ("q", "index".to_string()),
                ("limit", "1".to_string()),
                ("json", "1".to_string()),
                ("api_key", env::var("GELBOORU_API_KEY")?),
                ("user_id", env::var("GELBOORU_API_USER_ID")?),
                ("tags", tags),
            ]),
        })
    }

    fn url(&self) -> String {
        let mut url = self.pre_query.to_string() + "?";

        for (i, (k, v)) in self.query.iter().enumerate() {
            if i > 0 {
                url += "&";
            }
            url += k;
            url += "=";
            url += v;
        }

        url
    }
}
