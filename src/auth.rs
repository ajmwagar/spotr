const SPOTIFY_CLIENT_ID: &str = "f3a1096e3b9d43498c0a91fb713b65f1";
const SPOTIFY_SCOPES: &[&str] = &["user-read-private", "user-read-email"];
const SPOTIFY_REDIRECT_URL: &str = "http://localhost:8080";

const SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";
const SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/api/token";

use std::{collections::HashMap, error::Error};
use colored::Colorize;

use serde::{Serialize, Deserialize};

use oauth2::{AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, RefreshToken, RequestTokenError, Scope, TokenResponse, TokenUrl};

use oauth2::basic::BasicClient;
use oauth2::reqwest::async_http_client;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use url::Url;

use crate::config::{load_config, save_config, AuthTokens, Config};

fn get_oauth_client() -> Result<BasicClient, Box<dyn Error>> {
    // Create an OAuth2 client by specifying the client ID, client secret, authorization URL and
    // token URL.
    let client = BasicClient::new(
        ClientId::new(SPOTIFY_CLIENT_ID.to_string()),
        None,
        AuthUrl::new(SPOTIFY_AUTH_URL.to_string())?,
        Some(TokenUrl::new(SPOTIFY_TOKEN_URL.to_string())?),
    )
    // Set the URL the user will be redirected to after the authorization process.
    .set_redirect_uri(RedirectUrl::new(SPOTIFY_REDIRECT_URL.to_string())?);

    Ok(client)
}

pub async fn login() -> Result<(), Box<dyn Error>> {
    println!("{}", "Starting OAuth2.0 PKCE Flow".italic());
    let client = get_oauth_client()?;

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        // .add_scope(Scope::new("user-read-recently-played".to_string()))
        // .add_scope(Scope::new("user-read-playback-state".to_string()))
        // .add_scope(Scope::new("playlist-modify-private".to_string()))
        // .add_scope(Scope::new("playlist-read-private".to_string()))
        .add_scope(Scope::new("app-remote-control".to_string()))
        .add_scope(Scope::new("user-modify-playback-state".to_string()))
        // .add_scope(Scope::new("streaming".to_string()))
        // .add_scope(Scope::new("user-top-read".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Open URL in browser
    match open::that(auth_url.to_string()) {
        Ok(_) => println!("Opened {} in your browser.", "Spotify".green().bold()),
        Err(_) => eprintln!("Failed to open browser. Please browse to: {}", auth_url),
    }

    println!("{}", "Please complete the login flow in your browser.".italic().green());

    let pkce_verify_string = pkce_verifier.secret();
    // A very naive implementation of the redirect server.
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    loop {
        if let Ok((mut stream, _)) = listener.accept().await {
            let (code, state) = {
                let mut reader = BufReader::new(&mut stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line).await.unwrap();

                let redirect_url = request_line.split_whitespace().nth(1).unwrap();

                let uri_str = &(SPOTIFY_REDIRECT_URL.to_string() + redirect_url);

                let url = Url::parse(uri_str).unwrap();

                let code = if let Some(code_pair) = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    }) {
                    let (_, value) = code_pair;
                    let code = AuthorizationCode::new(value.into_owned());
                    Some(code)
                } else {
                    None
                };

                let state = if let Some(state_pair) = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    }) {
                    let (_, value) = state_pair;
                    let state = CsrfToken::new(value.into_owned());
                    Some(state)
                } else {
                    None
                };

                (code, state)
            };

            if let (Some(code), Some(state)) = (code, state) {
                let message = "Go back to your terminal :)";
                let response = format!(
                    "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                    message.len(),
                    message
                );
                stream.write_all(response.as_bytes()).await.unwrap();

                let (access_token, refresh_token) = get_access_token(&code, &PkceCodeVerifier::new(pkce_verify_string.clone())).await?;

                // Save token to config
                let config = Config {
                    auth: AuthTokens {
                        refresh_token: Some(refresh_token.secret().clone()),
                        access_token: Some(access_token.secret().clone()),
                    },
                    ..load_config()?
                };

                save_config(config)?;

                println!("{}", "Logged in!".green().bold());
                // The server will terminate itself after collecting the first code.
                break;
            }
        }
    }

    Ok(())
}

pub async fn refresh_token() -> Result<Config, Box<dyn Error>> {
    let config = load_config()?;

    if let Some(refresh_str) = config.auth.refresh_token {
        let http = reqwest::Client::new();
        let mut params = HashMap::new();
        
        params.insert("client_id", SPOTIFY_CLIENT_ID.to_string());
        params.insert("grant_type", "refresh_token".to_string());
        params.insert("refresh_token", refresh_str);

        let resp = http.post(SPOTIFY_TOKEN_URL).form(&params).send().await?;
        let tokens: Tokens = resp.json().await?;

        let config = Config {
            auth: AuthTokens {
                refresh_token: Some(tokens.refresh_token),
                access_token: Some(tokens.access_token),
            },
            ..config
        };

        save_config(config.clone())?;

        Ok(config)
    } else {
        Err(String::from("No refresh token in config. Please login using sp login.").into())
    }
}

pub async fn get_access_token(code: &AuthorizationCode, code_verifier: &PkceCodeVerifier) -> Result<(AccessToken, RefreshToken), Box<dyn Error>> {
    let http = reqwest::Client::new();

    let mut params = HashMap::new();
    
    params.insert("client_id", SPOTIFY_CLIENT_ID.to_string());
    params.insert("redirect_uri", SPOTIFY_REDIRECT_URL.to_string());
    params.insert("grant_type", "authorization_code".to_string());
    params.insert("code_verifier", code_verifier.secret().clone());
    params.insert("code", code.secret().clone());

    let resp = http.post(SPOTIFY_TOKEN_URL).form(&params).send().await?;
    let tokens: Tokens = resp.json().await?;

    Ok((AccessToken::new(tokens.access_token), RefreshToken::new(tokens.refresh_token)))
}

#[derive(Serialize, Deserialize)]
struct Tokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_refresh() {
        let result = refresh_token().await;
        println!("Result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_login() {
        let result = login().await;
        println!("Result: {:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_oauth_client() {
        let res = get_oauth_client();
        assert!(res.is_ok());
    }
}
