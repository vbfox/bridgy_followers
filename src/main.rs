use atrium_api::agent::atp_agent::{AtpAgent, store::MemorySessionStore};
use atrium_xrpc_client::reqwest::ReqwestClient;
use clap::Parser;
use color_eyre::Result;
use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;

use bsky::{get_known_followers, resolve_handle};

use crate::bsky::BskyAgent;
use crate::config::Config;

mod bsky;
mod config;
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

    // Check if Mastodon authentication exists, if not, authenticate
    let mastodon_config = match &config.mastodon {
        Some(cfg) => {
            println!("Using saved Mastodon authentication for {}", cfg.server);
            cfg.clone()
        }
        None => {
            println!("No saved Mastodon authentication found. Let's authenticate!\n");
            let mastodon_cfg = mastodon::authenticate().await?;
            config.mastodon = Some(mastodon_cfg.clone());
            config.save_to_file(&cli.config)?;
            println!("Mastodon credentials saved to config file.\n");
            mastodon_cfg
        }
    };

    // Get Mastodon following list
    let mastodon_following = mastodon::get_following(&mastodon_config).await?;

    let agent = create_authenticated_agent(&config).await?;

    let bridgy_did = resolve_handle(&agent, "ap.brid.gy").await?;
    let bridgy_followers = get_known_followers(&agent, &bridgy_did).await?;

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

        // Skip ignored accounts
        if config
            .ignored_accounts
            .iter()
            .any(|ignored| ignored == handle)
        {
            continue;
        }

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

    eprintln!("\nFiltered out {} accounts already followed on Mastodon", filtered_count);
    eprintln!("Total bridgy followers (after ignored): {}", total_count);
    eprintln!("New accounts to follow: {}", total_count - filtered_count);

    Ok(())
}

async fn create_authenticated_agent(config: &Config) -> Result<BskyAgent> {
    let agent = AtpAgent::new(
        ReqwestClient::new("https://bsky.social"),
        MemorySessionStore::default(),
    );

    agent.login(&config.username, &config.password).await?;

    Ok(agent)
}
