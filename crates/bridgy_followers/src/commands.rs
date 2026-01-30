use std::path::{Path, PathBuf};

use crate::bluesky::{self};
use crate::config::Config;
use crate::follower_status::{get_follower_statuses, statuses_to_import_csv};
use crate::{credentials, mastodon};
use color_eyre::Result;
use color_eyre::owo_colors::OwoColorize;

pub async fn sync_command(config_path: PathBuf, output_path: Option<PathBuf>) -> Result<()> {
    let mut config = Config::from_file(&config_path)?;

    let credential_builder = keyring::default::default_credential_builder();

    // Query the Mastodon user follows
    let mastodon_user = mastodon::authenticate(&credential_builder, &mut config).await?;
    let bluesky = bluesky::authenticate(&credential_builder, &mut config).await?;
    let statuses = get_follower_statuses(
        &mastodon_user,
        &bluesky,
        config.ignored_accounts(),
        output_path.is_none(),
    )
    .await?;

    let csv = statuses_to_import_csv(&statuses)?;
    println!("{}", csv);

    if let Some(output_path) = output_path {
        std::fs::write(&output_path, csv)?;
        println!("Wrote output to {}", output_path.display().blue());
    }

    Ok(())
}

pub async fn csv_command(config_path: PathBuf, output_path: Option<PathBuf>) -> Result<()> {
    let mut config = Config::from_file(&config_path)?;

    let credential_builder = keyring::default::default_credential_builder();

    // Query the Mastodon user follows
    let mastodon_user = mastodon::authenticate(&credential_builder, &mut config).await?;
    let bluesky = bluesky::authenticate(&credential_builder, &mut config).await?;
    let statuses =
        get_follower_statuses(&mastodon_user, &bluesky, config.ignored_accounts(), true).await?;

    let csv = statuses_to_import_csv(&statuses)?;
    println!("{}", csv);

    if let Some(output_path) = output_path {
        std::fs::write(&output_path, csv)?;
        println!("Wrote output to {}", output_path.display().blue());
    }

    Ok(())
}

pub fn forget_command(config_path: &Path) -> Result<()> {
    let mut config = Config::from_file(config_path)?;

    let credential_builder = keyring::default::default_credential_builder();

    // Get current values before clearing
    let bluesky_username = config.bluesky_username().map(ToString::to_string);
    let mastodon_server = config.mastodon_server().map(ToString::to_string);

    // Delete credentials from keyring
    credentials::delete_credentials(
        &credential_builder,
        mastodon_server.as_deref(),
        bluesky_username.as_deref(),
    );

    // Clear config values
    config.mutate(|mut data| {
        data.bluesky_username = None;
        data.mastodon_server = None;
        data
    })?;

    println!("Credentials cleared");

    Ok(())
}
