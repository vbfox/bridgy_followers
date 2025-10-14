use atrium_api::agent::atp_agent::{AtpAgent, store::MemorySessionStore};
use atrium_xrpc_client::reqwest::ReqwestClient;
use clap::Parser;
use color_eyre::{Result, Section};

use bsky::{get_known_followers, resolve_handle};

use crate::bsky::BskyAgent;

mod bsky;

#[derive(Parser)]
#[command(name = "bridgy_followers")]
#[command(about = "Find intersection of followers between a user and @ap.brid.gy")]
struct Cli {
    #[arg(help = "Bluesky user handle or DID")]
    user: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    let agent = create_authenticated_agent(cli.user.trim_start_matches('@')).await?;

    let bridgy_did = resolve_handle(&agent, "ap.brid.gy").await?;
    let bridgy_followers = get_known_followers(&agent, &bridgy_did).await?;
    println!("Account address,Show boosts,Notify on new posts,Languages");

    // For each user in intersection, get their handle and display Mastodon equivalent
    for follower in bridgy_followers.values() {
        let mastodon_handle = format!("@{}@bsky.brid.gy", follower.handle.as_str());
        println!("{mastodon_handle},true,false,");
    }
    Ok(())
}

async fn create_authenticated_agent(user: &str) -> Result<BskyAgent> {
    let agent = AtpAgent::new(
        ReqwestClient::new("https://bsky.social"),
        MemorySessionStore::default(),
    );

    let password = std::env::var("BSKY_PASSWORD")
        .suggestion("Ensure the BSKY_PASSWORD variable is defined")?;

    agent.login(user, &password).await?;

    Ok(agent)
}
