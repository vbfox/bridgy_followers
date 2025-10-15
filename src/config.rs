use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub ignored_accounts: Vec<String>,
    #[serde(default)]
    pub mastodon: Option<MastodonConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MastodonConfig {
    pub server: String,
    pub access_token: String,
}

impl Config {
    pub fn from_file(path: &Path) -> Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        // Read existing file and parse to preserve structure
        let contents = std::fs::read_to_string(path)?;
        let mut doc: toml::Value = toml::from_str(&contents)?;

        // Update mastodon section if present
        if let Some(mastodon) = &self.mastodon {
            let mut mastodon_table = toml::map::Map::new();
            mastodon_table.insert("server".to_string(), toml::Value::String(mastodon.server.clone()));
            mastodon_table.insert("access_token".to_string(), toml::Value::String(mastodon.access_token.clone()));

            if let toml::Value::Table(ref mut table) = doc {
                table.insert("mastodon".to_string(), toml::Value::Table(mastodon_table));
            }
        }

        let contents = toml::to_string_pretty(&doc)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}
