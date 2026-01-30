use color_eyre::{Result, eyre::eyre};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct ConfigData {
    pub bluesky_username: Option<String>,
    #[serde(default)]
    pub ignored_accounts: Vec<String>,
    pub mastodon_server: Option<String>,
}

pub struct Config {
    data: ConfigData,
    path: PathBuf,
}

impl Config {
    pub fn mastodon_server(&self) -> Option<&str> {
        self.data.mastodon_server.as_deref()
    }

    pub fn bluesky_username(&self) -> Option<&str> {
        self.data.bluesky_username.as_deref()
    }

    pub fn ignored_accounts(&self) -> &Vec<String> {
        &self.data.ignored_accounts
    }

    /// Load the configuration from a file
    pub fn from_file(path: &Path) -> Result<Self> {
        let data = match fs::read_to_string(path) {
            Ok(contents) => {
                let data: ConfigData = toml::from_str(&contents)?;
                data
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => ConfigData::default(),
            Err(e) => {
                return Err(e.into());
            }
        };
        Ok(Config {
            data,
            path: path.to_path_buf(),
        })
    }

    /// Save a new configuration to the configuration file
    pub fn mutate(&mut self, mutation: impl Fn(ConfigData) -> ConfigData) -> Result<()> {
        let new_data = mutation(self.data.clone());
        let toml_string = toml::to_string_pretty(&new_data)?;
        self.data = new_data;

        // Ensure the parent directory exists
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.path, toml_string)?;
        Ok(())
    }
}

/// Get the default configuration directory path
pub fn default_config_dir() -> Result<PathBuf> {
    ProjectDirs::from("net", "vbfox", "bridgy_followers")
        .map(|proj_dirs| proj_dirs.config_dir().to_path_buf())
        .ok_or_else(|| eyre!("Could not determine config directory"))
}

/// Get the default configuration file path
pub fn default_config_path() -> Result<PathBuf> {
    Ok(default_config_dir()?.join("config.toml"))
}
