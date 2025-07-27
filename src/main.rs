use atrium_api::agent::atp_agent::{AtpAgent, store::MemorySessionStore};
use atrium_xrpc_client::reqwest::ReqwestClient;
use clap::Parser;
use std::collections::HashSet;

#[derive(Parser)]
#[command(name = "bridgy_followers")]
#[command(about = "Find intersection of followers between a user and @ap.brid.gy")]
struct Cli {
    #[arg(help = "Bluesky user handle or DID")]
    user: String,
}

type MyAgent = AtpAgent<MemorySessionStore, ReqwestClient>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let agent = create_authenticated_agent().await?;

    // Resolve the input user to DID
    let user_did = resolve_handle(&agent, &cli.user).await?;
    println!("Resolved {} to DID: {}", cli.user, user_did);

    // Get followers for the target user
    let user_followers = get_all_followers(&agent, &user_did).await?;
    println!("Found {} followers for {}", user_followers.len(), cli.user);

    // Get followers for @ap.brid.gy
    let bridgy_did = resolve_handle(&agent, "ap.brid.gy").await?;
    let bridgy_followers = get_all_followers(&agent, &bridgy_did).await?;
    println!("Found {} followers for @ap.brid.gy", bridgy_followers.len());

    // Find intersection
    let intersection: HashSet<_> = user_followers.intersection(&bridgy_followers).collect();
    println!("\nFound {} users in common:", intersection.len());

    // For each user in intersection, get their handle and display Mastodon equivalent
    for did in intersection {
        if let Ok(handle) = get_handle_from_did(&agent, did).await {
            let mastodon_handle = format!("@{}@bsky.brid.gy", handle.trim_start_matches('@'));
            println!("{} -> {}", handle, mastodon_handle);
        }
    }

    Ok(())
}

async fn create_authenticated_agent() -> Result<MyAgent, Box<dyn std::error::Error>> {
    let agent = AtpAgent::new(
        ReqwestClient::new("https://bsky.social"),
        MemorySessionStore::default(),
    );

    let identifier = std::env::var("BSKY_IDENTIFIER")?;
    let password = std::env::var("BSKY_PASSWORD")?;

    agent.login(&identifier, &password).await?;

    Ok(agent)
}
async fn resolve_handle(
    agent: &MyAgent,
    handle: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    use atrium_api::com::atproto::identity::resolve_handle;

    let clean_handle = handle.trim_start_matches('@');

    // If it's already a DID, return as-is
    if clean_handle.starts_with("did:") {
        return Ok(clean_handle.to_string());
    }

    let params = resolve_handle::ParametersData {
        handle: clean_handle.parse()?,
    };

    let response = agent
        .api
        .com
        .atproto
        .identity
        .resolve_handle(params.into())
        .await?;
    Ok(response.data.did.to_string())
}

async fn get_all_followers(
    agent: &MyAgent,
    did: &str,
) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    use atrium_api::app::bsky::graph::get_followers;

    let mut all_followers = HashSet::new();
    let mut cursor = None;

    loop {
        let params = get_followers::ParametersData {
            actor: did.parse()?,
            cursor: cursor.clone(),
            limit: Some(100.try_into()?),
        };

        let response = agent
            .api
            .app
            .bsky
            .graph
            .get_followers(params.into())
            .await?;

        for follower in response.data.followers {
            all_followers.insert(follower.did.to_string());
        }

        if response.data.cursor.is_none() {
            break;
        }
        cursor = response.data.cursor;
    }

    Ok(all_followers)
}

async fn get_handle_from_did(
    agent: &MyAgent,
    did: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    use atrium_api::app::bsky::actor::get_profile;

    let params = get_profile::ParametersData {
        actor: did.parse()?,
    };

    let response = agent.api.app.bsky.actor.get_profile(params.into()).await?;
    Ok(response.data.handle.to_string())
}
