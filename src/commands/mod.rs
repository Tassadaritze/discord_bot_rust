use anyhow::Error;
use poise::Command;

use crate::DataWrapper;
use _8ball::_8ball;
use ping::ping;

#[path = "8ball.rs"]
mod _8ball;
mod ping;

pub fn commands() -> Vec<Command<DataWrapper, Error>> {
    vec![ping(), _8ball()]
}
