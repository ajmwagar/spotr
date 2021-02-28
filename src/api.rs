use std::{error::Error, time::Duration};

use colored::Colorize;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header::HeaderName, Client};
use serde_json::json;
use tokio::time::sleep;

use crate::model::{CurrentlyPlayingContext, PlayingItem};

use crate::{
    auth::refresh_token,
    config::{load_config, Config, Defaults},
};

const DELAY: u64 = 300;

pub async fn alias(config: Config) -> Result<(), Box<dyn Error>> {
    print!(
        r#"
    alias sps="sp skip"
    alias spb="sp back"
    alias spp="sp pause"
    alias spr="sp play"
    alias spc="sp current"
    "#
    );
    Ok(())
}

pub async fn play(config: Config) -> Result<(), Box<dyn Error>> {
    println!("Resuming playback on default device.");

    let base = "https://api.spotify.com/v1/me/player/play".to_string();
    
    let url = match &config.defaults.device {
        Some(dev) => format!("{}?device_id={}", base, dev),
        None => base
    };

    if let Some(playlist) = &config.defaults.playlist {
        let http = Client::new();
        let (key, val) = get_auth_header(&config)?;
        let req = http
            .put(&url)
            .header(key, val)
            .json(&json!({
                  "context_uri": playlist,
                  "offset": {
                    "position": 0
                  },
                  "position_ms": 0
            }));

        let resp = req.send().await?;
    }

    sleep(Duration::from_millis(DELAY)).await;

    current(config).await
}

pub async fn pause(config: Config) -> Result<(), Box<dyn Error>> {
    println!("Pausing playback.");

    let http = Client::new();
    let (key, val) = get_auth_header(&config)?;
    let req = http
        .put("https://api.spotify.com/v1/me/player/pause")
        .header(key, val)
        .json(&());

    let resp = req.send().await?;

    Ok(())
}

pub async fn skip(config: Config) -> Result<(), Box<dyn Error>> {
    println!("Skipping to next song.");

    let http = Client::new();
    let (key, val) = get_auth_header(&config)?;
    let req = http
        .post("https://api.spotify.com/v1/me/player/next")
        .header(key, val)
        .json(&());

    let resp = req.send().await?;

    sleep(Duration::from_millis(DELAY)).await;

    current(config).await
}

pub async fn back(config: Config) -> Result<(), Box<dyn Error>> {
    println!("Skipping to previous song.");

    let http = Client::new();
    let (key, val) = get_auth_header(&config)?;
    let req = http
        .post("https://api.spotify.com/v1/me/player/previous")
        .header(key, val)
        .json(&());

    let resp = req.send().await?;

    sleep(Duration::from_millis(DELAY)).await;

    current(config).await
}

pub async fn current(config: Config) -> Result<(), Box<dyn Error>> {
    let http = Client::new();

    let (key, val) = get_auth_header(&config)?;
    let req = http
        .get("https://api.spotify.com/v1/me/player/currently-playing")
        .header(key, val);

    let resp = req.send().await?;

    let json: Option<CurrentlyPlayingContext> = resp.json().await?;

    if let Some(json) = json {
        match json.item {
            Some(PlayingItem::Track(track)) => {
                if let Some(artist) = track.artists.first() {
                    println!(
                        "Currently playing {} by {}.",
                        track.name.green(),
                        artist.name.green()
                    )
                } else {
                    println!("Currently playing {}.", track.name.green());
                }
            }
            Some(PlayingItem::Episode(episode)) => {}
            _ => {}
        }
    } else {
        println!("Not currently playing.");
    }

    Ok(())
}

pub async fn save_to_playlist(config: Config) -> Result<(), Box<dyn Error>> {
    Ok(())
}

fn get_auth_header(config: &Config) -> Result<(HeaderName, HeaderValue), Box<dyn Error>> {
    if let Some(access_token) = &config.auth.access_token {
        Ok((
            "Authorization".parse()?,
            format!("Bearer {}", access_token).parse()?,
        ))
    } else {
        Err(String::from("No auth token!").into())
    }
}

mod models;
