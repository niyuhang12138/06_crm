use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{env, fs::File};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub pk: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // read from ./user-stat.yml or /etc/config/user-stat.yml or from env CHAT_CONFIG
        let ret = match (
            File::open("user-stat.yml"),
            File::open("/etc/config/user-stat.yml"),
            env::var("USER_STAT_CONFIG"),
        ) {
            (Ok(file), _, _) => serde_yaml::from_reader(file),
            (_, Ok(file), _) => serde_yaml::from_reader(file),
            (_, _, Ok(file)) => serde_yaml::from_str(&file),
            _ => bail!("Config file not found"),
        };

        Ok(ret?)
    }
}
