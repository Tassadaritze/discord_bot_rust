use anyhow::Error;
use poise::Command;

use crate::DataWrapper;
use _8ball::_8ball;
use aon::aon;
use gelbooru::gelbooru;
use ping::ping;
use register::register;
use shares::shares;

#[path = "8ball.rs"]
mod _8ball;
mod aon;
mod gelbooru;
mod ping;
mod register;
pub mod shares;

pub fn commands() -> Vec<Command<DataWrapper, Error>> {
    vec![ping(), _8ball(), register(), gelbooru(), aon(), shares()]
}
