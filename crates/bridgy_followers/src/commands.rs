use std::fs;
use std::path::{Path, PathBuf};

use crate::bluesky::{self};
use crate::config::Config;
use crate::follower_status::{FollowerStatus, get_follower_statuses, statuses_to_import_csv};
use crate::{credentials, mastodon, utils::bluesky_handle_to_mastodon};
use color_eyre::Result;
use color_eyre::owo_colors::OwoColorize;

pub async fn sync_command(config_path: PathBuf, _output_path: Option<PathBuf>) -> Result<()> {
    let mut config = Config::from_file(&config_path)?;

    let credential_builder = keyring::default::default_credential_builder();

    let mastodon_user = mastodon::authenticate(&credential_builder, &mut config).await?;
    let bluesky = bluesky::authenticate(&credential_builder, &mut config).await?;
    let statuses =
        get_follower_statuses(&mastodon_user, &bluesky, config.ignored_accounts(), false).await?;

    let ready_to_follow: Vec<_> = statuses
        .iter()
        .filter(|s| s.status == FollowerStatus::ReadyToFollow)
        .collect();

    if ready_to_follow.is_empty() {
        println!("{}", "No new accounts to follow!".green());
        return Ok(());
    }

    println!(
        "Found {} new account(s) to follow",
        ready_to_follow.len().yellow()
    );

    let mut success_count = 0;
    let mut error_count = 0;

    for follower in ready_to_follow {
        let mastodon_handle = bluesky_handle_to_mastodon(&follower.handle);
        print!("Following {}... ", format!("@{}", mastodon_handle).blue());

        match mastodon::follow_account(&mastodon_user, &mastodon_handle).await {
            Ok(_) => {
                println!("{}", "✓".green());
                success_count += 1;
            }
            Err(e) => {
                println!("{}", "✗".red());
                eprintln!("  Error: {}", e.to_string().red());
                error_count += 1;
            }
        }
    }

    println!();
    println!(
        "Successfully followed {} account(s)",
        success_count.to_string().green()
    );
    if error_count > 0 {
        println!(
            "Failed to follow {} account(s)",
            error_count.to_string().red()
        );
    }

    Ok(())
}

pub async fn csv_command(config_path: PathBuf, output_path: Option<PathBuf>) -> Result<()> {
    let mut config = Config::from_file(&config_path)?;

    let credential_builder = keyring::default::default_credential_builder();

    let mastodon_user = mastodon::authenticate(&credential_builder, &mut config).await?;
    let bluesky = bluesky::authenticate(&credential_builder, &mut config).await?;
    let statuses =
        get_follower_statuses(&mastodon_user, &bluesky, config.ignored_accounts(), true).await?;

    let csv = statuses_to_import_csv(&statuses)?;
    println!("{}", csv);

    if let Some(output_path) = output_path {
        fs::write(&output_path, csv)?;
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
