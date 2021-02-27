const SPOTIFY_CLIENT_ID: &str = "f3a1096e3b9d43498c0a91fb713b65f1";
const SPOTIFY_SCOPES: &[&str] = &["user-read-private", "user-read-email"];
const SPOTIFY_REDIRECT_URL: &str = "http://localhost:8080";

const SPOTIFY_AUTH_URL: &str = "https://accounts.spotify.com/authorize";
const SPOTIFY_TOKEN_URL: &str = "https://accounts.spotify.com/token";

use std::error::Error;

use oauth2::{
    AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    RedirectUrl, Scope, TokenResponse, TokenUrl,
};

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
    let client = get_oauth_client()?;

    // Generate a PKCE challenge.
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the full authorization URL.
    let (auth_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // Set the desired scopes.
        .add_scope(Scope::new("read".to_string()))
        .add_scope(Scope::new("write".to_string()))
        // Set the PKCE code challenge.
        .set_pkce_challenge(pkce_challenge)
        .url();

    // Open URL in browser
    match open::that(auth_url.to_string()) {
        Ok(_) => println!("Opened {} in your browser.", auth_url),
        Err(_) => eprintln!("Failed to open browser. Please browse to: {}", auth_url),
    }

    // A very naive implementation of the redirect server.
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    loop {
        if let Ok((mut stream, _)) = listener.accept().await {
            let code;
            let state;
            {
                let mut reader = BufReader::new(&mut stream);

                let mut request_line = String::new();
                reader.read_line(&mut request_line).await.unwrap();

                let redirect_url = request_line.split_whitespace().nth(1).unwrap();
                let url = Url::parse(&("http://localhost".to_string() + redirect_url)).unwrap();

                let code_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    })
                    .unwrap();

                let (_, value) = code_pair;
                code = AuthorizationCode::new(value.into_owned());

                let state_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    })
                    .unwrap();

                let (_, value) = state_pair;
                state = CsrfToken::new(value.into_owned());
            }

            let message = "Go back to your terminal :)";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
            );
            stream.write_all(response.as_bytes()).await.unwrap();

            println!("Spotify returned the following code:\n{}\n", code.secret());
            println!(
                "Spotify returned the following state:\n{} (expected `{}`)\n",
                state.secret(),
                csrf_state.secret()
            );

            // Exchange the code with a token.
            let token_res = client
                .exchange_code(code)
                .set_pkce_verifier(pkce_verifier)
                .request_async(async_http_client)
                .await;

            println!("Spotify returned the following token:\n{:?}\n", token_res);

            if let Ok(token) = token_res {
                let refresh_token = token.refresh_token();
                let access_token = token.access_token();
                let scopes = token.scopes();
                println!("Spotify returned the following scopes:\n{:?}\n", scopes);

                // Save token to config
                println!("Saving tokens.");
                let config = Config {
                    auth: AuthTokens {
                        refresh_token: match refresh_token {
                            Some(token) => Some(token.secret().clone()),
                            None => None,
                        },
                        access_token: Some(access_token.secret().clone()),
                    },
                    ..load_config()?
                };

                save_config(config)?;
            }

            // The server will terminate itself after collecting the first code.
            break;
        }
    }

    Ok(())
}

pub async fn refresh_token() -> Result<AccessToken, Box<dyn Error>> {
    let config = load_config()?;

    let client = get_oauth_client()?;

    // // Generate a PKCE challenge.
    // let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    if let Some(refresh_str) = config.auth.refresh_token {
        let refresh = oauth2::RefreshToken::new(refresh_str);

        let token_res = client
            .exchange_refresh_token(&refresh)
            .request_async(async_http_client)
            .await?;

        Ok(token_res.access_token().clone())
    }

    else {
        Err(String::from("No refresh token in config. Please login using sp login.").into())
    }
}
