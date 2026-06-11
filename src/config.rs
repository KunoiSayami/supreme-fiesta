use std::{collections::HashMap, path::Path};

use serde::Deserialize;

use crate::code::into_barcode;

#[derive(Deserialize)]
pub struct Platform {
    #[serde(alias = "api-key")]
    api_key: String,
    server: Option<String>,
}

impl Platform {
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn server(&self) -> Option<&String> {
        self.server.as_ref()
    }
}

#[derive(Deserialize, Default)]
pub struct Users {
    #[serde(flatten)]
    map: HashMap<i64, String>,
}

#[derive(Deserialize)]
pub struct Config {
    platform: Platform,
    #[serde(default)]
    users: Users,
}

impl Config {
    pub async fn load<P: AsRef<Path> + std::fmt::Debug>(file: P) -> anyhow::Result<Self> {
        let s = tokio::fs::read_to_string(file).await?;
        Ok(toml::from_str(&s)?)
    }

    pub fn platform(&self) -> &Platform {
        &self.platform
    }

    pub fn user_entries(&self) -> impl Iterator<Item = (i64, String)> + '_ {
        self.users.map.iter().map(|(&id, s)| (id, into_barcode(s)))
    }
}
