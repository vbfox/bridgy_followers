use super::utils::{BlueskyAgent, create_agent};
use crate::{
    config::{Config, ConfigData},
    credentials,
};
use color_eyre::{Result, eyre::Context};
use dialoguer::{Input, Password, theme::ColorfulTheme};
use keyring::CredentialBuilder;

/// Get the Bluesky username from config or prompt if not set
fn get_username(config: &mut Config) -> Result<String> {
    if let Some(server) = config.bluesky_username() {
        Ok(server.to_string())
    } else {
        let input: String = Input::new()
            .with_prompt("Bluesky username (e.g., user.bsky.social)")
            .interact_text()?;

        config.mutate(|data| ConfigData {
            bluesky_username: Some(input.clone()),
            ..data
        })?;
        Ok(input)
    }
}

/// Get the Bluesky password from the credential store or prompt if not set
fn get_password(credential_builder: &Box<CredentialBuilder>, username: &str) -> Result<String> {
    // TODO: We should use OAuth now that it's available in bluesky
    let credentials = credentials::get_bluesky_password(credential_builder, &username)?;

    match credentials.get_password() {
        Ok(password) => Ok(password),
        Err(keyring::Error::NoEntry) => {
            let password = Password::with_theme(&ColorfulTheme::default())
                .with_prompt("Bluesky Password")
                .interact()?;
            credentials.set_password(&password)?;
            Ok(password)
        }
        err @ Err(_) => err.wrap_err("Failed to access credential store"),
    }
}

pub async fn authenticate(
    credential_builder: &Box<CredentialBuilder>,
    config: &mut Config,
) -> Result<BlueskyAgent> {
    let username = get_username(config)?;
    let password = get_password(credential_builder, &username)?;

    create_agent(&username, &password).await
}
