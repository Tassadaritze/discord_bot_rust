use anyhow::Result;
use rand::{thread_rng, Rng};

use crate::Context;

/// Ask the Magic 8 Ball
#[poise::command(slash_command, rename = "8ball")]
pub async fn _8ball(
    ctx: Context<'_>,
    #[description = "Your question"] question: Option<String>,
) -> Result<()> {
    let random = thread_rng().gen_range(0..ANSWERS.len());
    if let Some(question) = question {
        ctx.say(format!("```{question}```\n{}", ANSWERS[random]))
            .await?;
    } else {
        ctx.say(ANSWERS[random]).await?;
    }

    Ok(())
}

const ANSWERS: [&str; 20] = [
    "It is certain.",
    "It is decidedly so.",
    "Without a doubt.",
    "Yes definitely.",
    "You may rely on it.",
    "As I see it, yes.",
    "Most likely.",
    "Outlook good.",
    "Yes.",
    "Signs point to yes.",
    "Reply hazy, try again.",
    "Ask again later.",
    "Better not tell you now.",
    "Cannot predict now.",
    "Concentrate and ask again.",
    "Don't count on it.",
    "My reply is no.",
    "My sources say no.",
    "Outlook not so good.",
    "Very doubtful.",
];
