use atrium_api::{
    agent::atp_agent::{AtpAgent, store::MemorySessionStore},
    app::bsky::actor::defs::ProfileViewData,
    types::{Object, string::Did},
};
use atrium_xrpc_client::reqwest::ReqwestClient;
use color_eyre::{Result, eyre::eyre};
use std::collections::HashMap;

pub type BskyAgent = AtpAgent<MemorySessionStore, ReqwestClient>;

pub async fn resolve_handle(agent: &BskyAgent, handle: &str) -> Result<String> {
    use atrium_api::com::atproto::identity::resolve_handle;

    let clean_handle = handle.trim_start_matches('@');

    // If it's already a DID, return as-is
    if clean_handle.starts_with("did:") {
        return Ok(clean_handle.to_string());
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
    Ok(response.data.did.to_string())
}

pub async fn get_known_followers(
    agent: &BskyAgent,
    did: &str,
) -> Result<HashMap<Did, Object<ProfileViewData>>> {
    use atrium_api::app::bsky::graph::get_known_followers;

    let mut all_followers = HashMap::new();
    let mut cursor = None;

    loop {
        let params = get_known_followers::ParametersData {
            actor: did
                .parse()
                .map_err(|e| eyre!("Failed to parse DID: {}", e))?,
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

