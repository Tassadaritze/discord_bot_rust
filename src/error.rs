use anyhow::{bail, Error, Result};
use log::{debug, error};
use poise::FrameworkError;

use crate::commands::gelbooru::GelbooruError;
use crate::DataWrapper;

pub async fn handle_error(e: FrameworkError<'_, DataWrapper, Error>) -> Result<()> {
    match e {
        FrameworkError::Command { ctx, error } => {
            if ctx.framework().options.commands.contains(ctx.command()) {
                let response = match error.downcast::<GelbooruError>() {
                    Ok(e) => e.to_string(),
                    Err(e) => {
                        debug!("{:?}", e);
                        "An error occurred while executing this command.".to_string()
                    }
                };
                if let Err(e) = ctx.say(response).await {
                    error!("could not reply in on_error: {e}");
                };
            }
        }
        FrameworkError::Setup { error, .. } => {
            bail!(error);
        }
        _ => bail!("unhandled error"),
    }

    Ok(())
}
