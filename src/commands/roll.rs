use anyhow::Result;
use nom::branch::alt;
use nom::character::complete::digit1;
use nom::combinator::map;
use nom::number::complete::double;
use nom::{error_position, IResult};

use crate::Context;

/// Roll dice
#[poise::command(slash_command)]
pub async fn roll(ctx: Context<'_>, #[description = "What to roll"] roll: String) -> Result<()> {
    Ok(())
}

enum Token {
    Int(u64),
    Float(f64),
    Die,
    KeepHighest,
    KeepLowest,
    Add,
    Sub,
    Mul,
    Div,
}

fn num(i: &str) -> IResult<&str, Token> {
    alt((
        map(double, Token::Float),
        map(digit1, |i: &str| match i.parse() {
            Ok(result) => return Token::Int(result),
            Err(_) => error_position!(nom::error::ErrorKind::Digit),
        }),
    ))(i)
}
