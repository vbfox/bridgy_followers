use atrium_api::{
    agent::atp_agent::{AtpAgent, store::MemorySessionStore},
    app::bsky::actor::defs::ProfileViewData,
    types::{Object, string::Did},
};
use atrium_xrpc_client::reqwest::ReqwestClient;
use clap::Parser;
use color_eyre::{eyre::eyre, Result, Section};
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "bridgy_followers")]
#[command(about = "Find intersection of followers between a user and @ap.brid.gy")]
struct Cli {
    #[arg(help = "Bluesky user handle or DID")]
    user: String,
}

type MyAgent = AtpAgent<MemorySessionStore, ReqwestClient>;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = Cli::parse();
    println!("Starting...");

    let agent = create_authenticated_agent(&cli.user).await?;
    println!("Successfully authenticated {}", cli.user);

    // Resolve the input user to DID
    let user_did = resolve_handle(&agent, &cli.user).await?;
    println!("Resolved {} to DID: {}", cli.user, user_did);

    // Get followers for @ap.brid.gy
    let bridgy_did = resolve_handle(&agent, "ap.brid.gy").await?;
    let bridgy_followers = get_known_followers(&agent, &bridgy_did).await?;
    println!("Found {} followers for @ap.brid.gy", bridgy_followers.len());
    println!();
    println!();
    println!("Account address,Show boosts,Notify on new posts,Languages");

    // For each user in intersection, get their handle and display Mastodon equivalent
    for follower in bridgy_followers.values() {
        let mastodon_handle = format!("@{}@bsky.brid.gy", follower.handle.as_str());
        println!("{},true,false,", mastodon_handle);
    }
    Ok(())
}

async fn create_authenticated_agent(user: &str) -> Result<MyAgent> {
    let agent = AtpAgent::new(
        ReqwestClient::new("https://bsky.social"),
        MemorySessionStore::default(),
    );

    let password = std::env::var("BSKY_PASSWORD").suggestion("Ensure the BSKY_PASSWORD variable is defined")?;

    agent.login(user, &password).await?;

    Ok(agent)
}
async fn resolve_handle(
    agent: &MyAgent,
    handle: &str,
) -> Result<String> {
    use atrium_api::com::atproto::identity::resolve_handle;

    let clean_handle = handle.trim_start_matches('@');

    // If it's already a DID, return as-is
    if clean_handle.starts_with("did:") {
        return Ok(clean_handle.to_string());
    }

    let params = resolve_handle::ParametersData {
        handle: clean_handle.parse().map_err(|e| eyre!("Failed to parse handle: {}", e))?,
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

async fn get_known_followers(
    agent: &MyAgent,
    did: &str,
) -> Result<HashMap<Did, Object<ProfileViewData>>> {
    use atrium_api::app::bsky::graph::get_known_followers;

    let mut all_followers = HashMap::new();
    let mut cursor = None;

    loop {
        let params = get_known_followers::ParametersData {
            actor: did.parse().map_err(|e| eyre!("Failed to parse DID: {}", e))?,
            cursor: cursor.clone(),
            limit: Some(100.try_into().map_err(|e| eyre!("Failed to convert limit: {}", e))?),
        };

        let response = agent
            .api
            .app
            .bsky
            .graph
            .get_known_followers(params.into())
            .await?;

        for follower in response.data.followers {
            all_followers.insert(follower.did.clone(), follower);
        }

        if response.data.cursor.is_none() {
            break;
        }
        cursor = response.data.cursor;
    }

    Ok(all_followers)
}
