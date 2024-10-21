use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Platform {
    #[serde(alias = "api-key")]
    api_key: String,
    owner: i64,
    server: Option<String>,
}

impl Platform {
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn owner(&self) -> i64 {
        self.owner
    }

    pub fn server(&self) -> Option<&String> {
        self.server.as_ref()
    }
}

#[derive(Deserialize)]
pub struct Config {
    platform: Platform,
    id: String,
}

impl Config {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub async fn load<P: AsRef<Path> + std::fmt::Debug>(file: P) -> anyhow::Result<Self> {
        let s = tokio::fs::read_to_string(file).await?;
        Ok(toml::from_str(&s)?)
    }

    pub fn platform(&self) -> &Platform {
        &self.platform
    }

    pub fn barcode_id(&self) -> String {
        format!("\u{00C0}{}", self.id())
    }
}
