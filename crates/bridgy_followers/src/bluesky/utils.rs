use atrium_api::{
    agent::atp_agent::{AtpAgent, store::MemorySessionStore},
    app::bsky::actor::defs::ProfileViewData,
    types::{Object, string::Did},
};
use atrium_xrpc_client::reqwest::ReqwestClient;
use color_eyre::{Result, eyre::eyre};
use std::collections::HashMap;

pub type BlueskyAgent = AtpAgent<MemorySessionStore, ReqwestClient>;

const BRIDGY_HANDLE: &str = "ap.brid.gy";

pub async fn create_agent(username: &str, password: &str) -> Result<BlueskyAgent> {
    let agent = AtpAgent::new(
        ReqwestClient::new("https://bsky.social"),
        MemorySessionStore::default(),
    );

    agent.login(username, password).await?;

    Ok(agent)
}

/// Resolve a Bluesky handle or DID to a DID
pub async fn resolve_handle(agent: &BlueskyAgent, handle: &str) -> Result<Did> {
    use atrium_api::com::atproto::identity::resolve_handle;

    let clean_handle = handle.trim_start_matches('@');

    // If it's already a DID, return as-is
    if clean_handle.starts_with("did:") {
        return Did::new(clean_handle.to_string()).map_err(|e| eyre!("Failed to parse DID: {e}"));
    }

    let params = resolve_handle::ParametersData {
        handle: clean_handle
            .parse()
            .map_err(|e| eyre!("Failed to parse handle: {}", e))?,
    };

    let response = agent
        .api
        .com
        .atproto
        .identity
        .resolve_handle(params.into())
        .await?;
    Ok(response.data.did)
}

pub async fn get_bridgy_did(agent: &BlueskyAgent) -> Result<Did> {
    resolve_handle(&agent, BRIDGY_HANDLE).await
}

/// Enumerates accounts which follow a specified account (actor) and are followed by the viewer.
pub async fn get_known_followers(
    agent: &BlueskyAgent,
    did: &Did,
) -> Result<HashMap<Did, Object<ProfileViewData>>> {
    use atrium_api::app::bsky::graph::get_known_followers;

    let mut all_followers = HashMap::new();
    let mut cursor = None;

    loop {
        let params = get_known_followers::ParametersData {
            actor: did.clone().into(),
            cursor: cursor.clone(),
            limit: Some(
                100.try_into()
                    .map_err(|e| eyre!("Failed to convert limit: {}", e))?,
            ),
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

pub async fn get_known_bridgy_followers(
    agent: &BlueskyAgent,
    ignored_accounts: &Vec<String>,
) -> Result<HashMap<Did, Object<ProfileViewData>>> {
    let bridgy_did = get_bridgy_did(&agent).await?;
    let mut bridgy_followers = get_known_followers(&agent, &bridgy_did).await?;

    bridgy_followers.retain(|_, profile| {
        let handle = profile.handle.as_str();
        let ignored = ignored_accounts.iter().any(|ignored| ignored == handle);
        !ignored
    });
    Ok(bridgy_followers)
}
