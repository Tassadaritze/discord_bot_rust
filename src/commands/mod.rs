use anyhow::Error;
use poise::Command;

use crate::DataWrapper;

mod ping;

pub fn commands() -> Vec<Command<DataWrapper, Error>> {
    vec![ping::ping()]
}
