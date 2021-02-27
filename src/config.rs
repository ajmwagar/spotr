use serde::{Serialize, Deserialize};

const CRATE_NAME: &str = "spotr";

#[derive(Serialize, Deserialize, Default)]
pub struct Config {
    pub auth: AuthTokens,
    pub defaults: Defaults
}

#[derive(Serialize, Deserialize, Default)]
pub struct AuthTokens {
    pub refresh_token: Option<String>,
    pub access_token: Option<String>
}

#[derive(Serialize, Deserialize, Default)]
pub struct Defaults {
    pub playlist: Option<String>,
    pub device: Option<String>
}


pub fn load_config() -> Result<Config, confy::ConfyError> {
    let config: Config = confy::load(CRATE_NAME)?;
    Ok(config)
}

pub fn save_config(config: Config) -> Result<(), confy::ConfyError> {
    confy::store(CRATE_NAME, config)?;

    Ok(())
}
