use std::str::FromStr;

use serde::{de::Error, Deserialize, Deserializer};
use strum_macros::{Display, EnumString};

#[derive(serde::Deserialize)]
pub struct Config {
    pub network: NetworkConfig,
}

#[derive(serde::Deserialize)]
pub struct NetworkConfig {
    #[serde(deserialize_with = "deserialize_host")]
    pub host: [u8; 4],
    pub port: u16,
}

/// Custom de-serialiser for the host, converting a string value to `[u8; 4]`
fn deserialize_host<'de, D>(deserializer: D) -> Result<[u8; 4], D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let split: Vec<&str> = s.split('.').collect();

    if split.len() != 4 {
        return Err(D::Error::custom(
            "Invalid host -> value needs to be provided in the format '0.0.0.0', with 4 period-separated numbers between 0 and 255.",
        ));
    }

    let mut res: [u8; 4] = [0, 0, 0, 0];
    for (i, s) in split.into_iter().enumerate() {
        res[i] = match s.parse::<u8>() {
            Ok(n) => n,
            Err(e) => {
                return Err(D::Error::custom(format!(
                    "Invalid host -> error parsing one of the period-separated numbers for the host - ensure all values are within 0-255.\n\
                    Error encountered: {e}"
                )))
            }
        };
    }

    Ok(res)
}

#[derive(Display, EnumString)]
pub enum Environment {
    #[strum(ascii_case_insensitive, to_string = "dev")]
    Dev,
    #[strum(ascii_case_insensitive, to_string = "prod")]
    Prod,
}

/// Resolve hierarchical configuration from config files and environment variables
pub fn get_config() -> Result<Config, config::ConfigError> {
    let config_dir = std::env::current_dir()
        .expect("Failed to determine the current directory")
        .join("config");

    let env: Environment = std::env::var("APP_ENV")
        .map(|e| Environment::from_str(&e).expect("Failed to parse `APP_ENV`"))
        .unwrap_or(Environment::Dev);

    config::Config::builder()
        // Config files
        .add_source(config::File::from(config_dir.join("base.toml")))
        .add_source(config::File::from(config_dir.join(format!("{}.toml", env))))
        // Environment variables e.g. `APP_NETWORK__HOST`
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()?
        .try_deserialize::<Config>()
}

#[cfg(test)]
mod tests {
    use super::*;

    // UNIT TESTS - HELPERS
    #[test]
    fn test_get_config() {
        assert!(get_config().is_ok());
    }
}
