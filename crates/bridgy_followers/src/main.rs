use crate::bluesky::get_known_bridgy_followers;
use crate::config::Config;
use clap::Parser;
use color_eyre::Result;
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
struct Cli {
    #[arg(
        default_value = "bridgy_followers.toml",
        help = "Path to configuration file"
    )]
    config: PathBuf,

    #[arg(short, long, help = "Output file (defaults to stdout)")]
    output: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let mut config = Config::from_file(&cli.config)?;

    let credential_builder = keyring::default::default_credential_builder();
    let mastodon = mastodon::authenticate(&credential_builder, &mut config).await?;
    let bluesky = bluesky::authenticate(&credential_builder, &mut config).await?;
    let mastodon_following = mastodon::get_following(&mastodon).await?;

    let bridgy_followers = get_known_bridgy_followers(&bluesky, &config.ignored_accounts()).await?;

    // Open output destination
    let mut output: Box<dyn Write> = match cli.output {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(io::stdout()),
    };

    writeln!(
        output,
        "Account address,Show boosts,Notify on new posts,Languages"
    )?;

    let mut filtered_count = 0;
    let mut total_count = 0;

    // For each user in intersection, get their handle and display Mastodon equivalent
    for follower in bridgy_followers.values() {
        let handle = follower.handle.as_str();

        total_count += 1;

        // Check if already following on Mastodon (handle@bsky.brid.gy format)
        let mastodon_handle_check = format!("{}@bsky.brid.gy", handle);
        if mastodon_following.contains(&mastodon_handle_check.to_lowercase()) {
            filtered_count += 1;
            continue;
        }

        let mastodon_handle = format!("@{}@bsky.brid.gy", handle);
        writeln!(output, "{mastodon_handle},true,false,")?;
    }

    eprintln!(
        "\nFiltered out {} accounts already followed on Mastodon",
        filtered_count
    );
    eprintln!("Total bridgy followers (after ignored): {}", total_count);
    eprintln!("New accounts to follow: {}", total_count - filtered_count);

    Ok(())
}
