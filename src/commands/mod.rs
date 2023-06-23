use anyhow::Error;
use poise::Command;

use crate::DataWrapper;
use _8ball::_8ball;
use aon::aon;
use gelbooru::gelbooru;
use ping::ping;
use register::register;
use roll::roll;
use shares::shares;

#[path = "8ball.rs"]
mod _8ball;
mod aon;
pub mod gelbooru;
mod ping;
mod register;
mod roll;
pub mod shares;

pub fn commands() -> Vec<Command<DataWrapper, Error>> {
    vec![
        ping(),
        _8ball(),
        register(),
        gelbooru(),
        aon(),
        shares(),
        roll(),
    ]
}
