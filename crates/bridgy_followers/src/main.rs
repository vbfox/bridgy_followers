#![allow(clippy::borrowed_box, reason = "Trigger on &Box<dyn Trait> parameters")]

use crate::bluesky::{get_bridgy_did, get_known_followers, get_relationships};
use crate::config::Config;
use atrium_api::types::string::Handle;
use clap::Parser;
use color_eyre::Result;
use ipld_core::ipld::Ipld;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

mod bluesky;
mod config;
mod credentials;
mod mastodon;

#[derive(Parser)]
#[command(name = "bridgy_followers")]
#[command(about = "Find intersection of followers between a user and @ap.brid.gy")]
#[command(args_conflicts_with_subcommands = true)]
struct CliArgs {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    /// Sync followers from Bluesky to Mastodon (default)
    Sync {
        #[arg(
            default_value = "bridgy_followers.toml",
            help = "Path to configuration file"
        )]
        config: PathBuf,

        #[arg(short, long, help = "Output file (defaults to stdout)")]
        output: Option<PathBuf>,
    },
    /// Clear stored credentials and configuration
    Forget {
        #[arg(
            default_value = "bridgy_followers.toml",
            help = "Path to configuration file"
        )]
        config: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = CliArgs::parse();

    match cli.command {
        Command::Sync { config, output } => sync_command(config, output).await,
        Command::Forget { config } => forget_command(config).await,
    }
}

fn bluesky_handle_to_mastodon(handle: &Handle) -> String {
    format!("{}@bsky.brid.gy", handle.as_str())
}

async fn sync_command(config_path: PathBuf, output_path: Option<PathBuf>) -> Result<()> {
    let mut config = Config::from_file(&config_path)?;

    let credential_builder = keyring::default::default_credential_builder();

    // Query the Mastodon user follows
    let mastodon = mastodon::authenticate(&credential_builder, &mut config).await?;
    let mastodon_following = mastodon::get_following(&mastodon).await?;

    // Query the intersection of our follows and the bridgy follows
    let bluesky = bluesky::authenticate(&credential_builder, &mut config).await?;
    let bridgy_did = get_bridgy_did(&bluesky).await?;
    let bridgy_followers = get_known_followers(&bluesky, &bridgy_did).await?;

    // Filter out ignored accounts and those already followed on Mastodon
    let ignored_accounts = config.ignored_accounts();
    let potential_new_follows: Vec<_> = bridgy_followers
        .values()
        .filter(|follower| {
            let ignored = ignored_accounts
                .iter()
                .any(|handle| handle == follower.handle.as_str());
            if ignored {
                return false;
            }

            let mastodon_handle = bluesky_handle_to_mastodon(&follower.handle).to_lowercase();
            let already_following = mastodon_following.contains(&mastodon_handle);
            if already_following {
                return false;
            }

            true
        })
        .collect();

    // We check if the bridge follow the users on Bluesky, this check is cheap but incomplete as in some cases where
    // the user bridged at some point but no longer it's not good enough. To be sure we need to check that the user
    // still follow the bridge.
    let relationships = get_relationships(
        &bluesky,
        bridgy_did.clone().into(),
        potential_new_follows.iter().map(|f| f.did.clone().into()),
    )
    .await?;

    let new_follows: Vec<_> = potential_new_follows
        .into_iter()
        .filter(|follow| {
            match relationships.get(&follow.did) {
                None => false,
                Some(relationship) => {
                    // Blocks are a Recent adition to the Lexicon, not yet in atrium
                    // https://github.com/bluesky-social/atproto/pull/4418
                    let extra_data: BTreeMap<String, Ipld> = relationship
                        .extra_data
                        .clone()
                        .try_into()
                        .unwrap_or_default();
                    let blocks_bridge = extra_data.contains_key("blockedBy")
                        || extra_data.contains_key("blockedByList");
                    if blocks_bridge {
                        println!(
                            "Skipping {} as they block the bridge",
                            follow.handle.as_str()
                        );
                        return false;
                    }

                    // TODO: Found out why there are weird cases;
                    // - '@thornbulle.bsky.social' doesn't report following the bridge but is bridged
                    // - '@terribletoybox.com@bsky.brid.gy' follows both way but is not bridged
                    // - '@pwnallthethings.bsky.social@bsky.brid.gy' same

                    true
                }
            }
        })
        .collect();

    // Open output destination
    let mut output: Box<dyn Write> = match output_path {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(io::stdout()),
    };

    writeln!(
        output,
        "Account address,Show boosts,Notify on new posts,Languages"
    )?;

    // For each user in intersection, get their handle and display Mastodon equivalent
    for follower in new_follows.iter() {
        let mastodon_handle = bluesky_handle_to_mastodon(&follower.handle);
        writeln!(output, "@{mastodon_handle},true,false,")?;
    }

    Ok(())
}

async fn forget_command(config_path: PathBuf) -> Result<()> {
    let mut config = Config::from_file(&config_path)?;

    let credential_builder = keyring::default::default_credential_builder();

    // Get current values before clearing
    let bluesky_username = config.bluesky_username().map(|s| s.to_string());
    let mastodon_server = config.mastodon_server().map(|s| s.to_string());

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
