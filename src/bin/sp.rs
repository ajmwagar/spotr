use std::error::Error;

use clap::Clap;
use spotr::{api::{alias, back, current, pause, play, skip}, args::{Opts, SubCommand}, auth::login, config::load_config};

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();

    let mut config = match load_config() {
        Ok(config) => config,
        Err(why) => {
            eprintln!("Cannot find config file: {:?}", why);
            return;
        }
    };

    match opts.subcmd {
        SubCommand::Login => {
            let _ = login().await;
        }
        subcmd => {
            if let Some(_refresh_token_str) = &config.auth.refresh_token {
                config = match spotr::auth::refresh_token().await {
                    Ok(config) => config,
                    Err(why) => {
                        eprintln!("Failed to refresh access token from Spotify API: {}.", why);
                        return;
                    }
                };
            } else {
                eprintln!("You're not logged in, please use sp login.");
                return;
            }

            if let Err(result) = match subcmd {
                SubCommand::Play => play(config).await,
                SubCommand::Pause => pause(config).await,
                SubCommand::Current => current(config).await,
                SubCommand::Skip => skip(config).await,
                SubCommand::Back => back(config).await,
                SubCommand::Alias => alias(config).await,
                _ => Ok(()),
            } {
                eprintln!("Error occured: {}", result);
            }
        }
    }
}
