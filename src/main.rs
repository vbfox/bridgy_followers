use atrium_api::agent::atp_agent::{AtpAgent, store::MemorySessionStore};
use atrium_xrpc_client::reqwest::ReqwestClient;
use clap::Parser;
use color_eyre::Result;
use std::path::PathBuf;

use bsky::{get_known_followers, resolve_handle};

use crate::bsky::BskyAgent;
use crate::config::Config;

mod bsky;
mod config;

#[derive(Parser)]
#[command(name = "bridgy_followers")]
#[command(about = "Find intersection of followers between a user and @ap.brid.gy")]
struct Cli {
    #[arg(
        default_value = "bridgy_followers.toml",
        help = "Path to configuration file"
    )]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let config = Config::from_file(&cli.config)?;

    let agent = create_authenticated_agent(&config).await?;

    let bridgy_did = resolve_handle(&agent, "ap.brid.gy").await?;
    let bridgy_followers = get_known_followers(&agent, &bridgy_did).await?;
    println!("Account address,Show boosts,Notify on new posts,Languages");

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

        let mastodon_handle = format!("@{}@bsky.brid.gy", handle);
        println!("{mastodon_handle},true,false,");
    }
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
