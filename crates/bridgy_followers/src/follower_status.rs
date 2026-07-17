use crate::{
    bluesky::{BlueskyAgent, get_bridgy_did, get_known_followers, get_relationships},
    mastodon,
    utils::{BRIDGY_ACTIVITY_PUB_URL, bluesky_handle_to_mastodon},
    webfinger,
};
use atrium_api::{
    app::bsky::actor::defs::ProfileViewData,
    types::{
        Object,
        string::{Did, Handle},
    },
};
use color_eyre::Result;
use ipld_core::ipld::Ipld;
use megalodon::mastodon::Mastodon;
use std::collections::{BTreeMap, HashSet};
use std::io;
use tracing::info;

/// Represents a bridged follower with their current status
#[derive(Debug, Clone)]
pub struct BridgedFollower {
    pub handle: Handle,
    pub status: FollowerStatus,
}

impl BridgedFollower {
    pub fn new(handle: Handle, status: FollowerStatus) -> Self {
        Self { handle, status }
    }
}

/// The status of a bridged follower
#[derive(Debug, Clone, PartialEq)]
pub enum FollowerStatus {
    /// User is in the ignored accounts list
    Ignored,
    /// User is already followed on Mastodon
    AlreadyFollowedOnMastodon,
    /// User is ready to be followed
    ReadyToFollow,
    /// User is not bridged
    NotBridged(NotBridgedReason),
}

/// Specific reason why we found out that a user is not bridged
#[derive(Debug, Clone, PartialEq)]
pub enum NotBridgedReason {
    /// User blocks the Bridgy bridge account
    BlocksBridge,
    /// No relationship data available from API
    NoRelationshipData,
    // The webfinger lookup on bridgy returned no account
    NoAccountOnBridgy,
}

fn filter_ignored_or_already_following<'a>(
    to_process: impl Iterator<Item = &'a Object<ProfileViewData>>,
    ignored_accounts: &[String],
    mastodon_following: &HashSet<String>,
    result: &mut Vec<BridgedFollower>,
) -> Vec<&'a Object<ProfileViewData>> {
    to_process
        .filter(|bsky_user| {
            let ignored = ignored_accounts
                .iter()
                .any(|handle| handle == bsky_user.handle.as_str());
            if ignored {
                info!(
                    did = bsky_user.did.as_str(),
                    "User '{}' in ignore list",
                    bsky_user.handle.as_str()
                );
                result.push(BridgedFollower::new(
                    bsky_user.handle.clone(),
                    FollowerStatus::Ignored,
                ));
                return false;
            }

            let mastodon_handle = bluesky_handle_to_mastodon(&bsky_user.handle);
            let already_following = mastodon_following.contains(&mastodon_handle);
            if already_following {
                info!(
                    did = bsky_user.did.as_str(),
                    "User '{}' already followed on Mastodon as {mastodon_handle}",
                    bsky_user.handle.as_str()
                );
                result.push(BridgedFollower::new(
                    bsky_user.handle.clone(),
                    FollowerStatus::AlreadyFollowedOnMastodon,
                ));
                return false;
            }

            true
        })
        .collect()
}

async fn filter_blocking_bridge<'a>(
    bluesky: &BlueskyAgent,
    bridgy_did: &Did,
    to_process: Vec<&'a Object<ProfileViewData>>,
    result: &mut Vec<BridgedFollower>,
) -> Result<Vec<&'a Object<ProfileViewData>>> {
    let relationships = get_relationships(
        bluesky,
        bridgy_did.clone().into(),
        to_process.iter().map(|f| f.did.clone().into()),
    )
    .await?;

    Ok(to_process
        .into_iter()
        .filter(|bsky_user| {
            match relationships.get(&bsky_user.did) {
                None => {
                    info!(
                        did = bsky_user.did.as_str(),
                        "User '{}' has no relationship with the bridge",
                        bsky_user.handle.as_str()
                    );
                    result.push(BridgedFollower::new(
                        bsky_user.handle.clone(),
                        FollowerStatus::NotBridged(NotBridgedReason::NoRelationshipData),
                    ));
                    false
                }
                Some(relationship) => {
                    // Blocks are a Recent addition to the Lexicon, not yet in atrium
                    // https://github.com/bluesky-social/atproto/pull/4418
                    let extra_data: BTreeMap<String, Ipld> = relationship
                        .extra_data
                        .clone()
                        .try_into()
                        .unwrap_or_default();
                    let blocks_bridge = extra_data.contains_key("blockedBy")
                        || extra_data.contains_key("blockedByList");

                    let followed_by_bridge = relationship.followed_by.is_some();

                    if blocks_bridge {
                        info!(
                            ?followed_by_bridge,
                            ?blocks_bridge,
                            did = bsky_user.did.as_str(),
                            "User '{}' blocks the bridge, filtering",
                            bsky_user.handle.as_str()
                        );
                        result.push(BridgedFollower::new(
                            bsky_user.handle.clone(),
                            FollowerStatus::NotBridged(NotBridgedReason::BlocksBridge),
                        ));
                        return false;
                    }

                    info!(
                        ?followed_by_bridge,
                        ?blocks_bridge,
                        did = bsky_user.did.as_str(),
                        "Need to add new user '{}'",
                        bsky_user.handle.as_str()
                    );
                    true
                }
            }
        })
        .collect())
}

async fn check_webfinger_bridging(
    to_process: Vec<&Object<ProfileViewData>>,
    result: &mut Vec<BridgedFollower>,
) -> Result<()> {
    for bsky_user in to_process {
        let mastodon_handle = bluesky_handle_to_mastodon(&bsky_user.handle);

        let account_exists =
            webfinger::account_exists(BRIDGY_ACTIVITY_PUB_URL, &mastodon_handle).await?;
        if account_exists {
            info!(
                did = bsky_user.did.as_str(),
                "User '{}' is bridged and ready to follow",
                bsky_user.handle.as_str()
            );
            result.push(BridgedFollower::new(
                bsky_user.handle.clone(),
                FollowerStatus::ReadyToFollow,
            ));
        } else {
            info!(
                did = bsky_user.did.as_str(),
                "User '{}' not found on bridgy webfinger",
                bsky_user.handle.as_str()
            );
            result.push(BridgedFollower::new(
                bsky_user.handle.clone(),
                FollowerStatus::NotBridged(NotBridgedReason::NoAccountOnBridgy),
            ));
        }
    }

    Ok(())
}


pub async fn get_follower_statuses(
    mastodon_user: &Mastodon,
    bluesky: &BlueskyAgent,
    ignored_accounts: &[String],
    quiet: bool,
) -> Result<Vec<BridgedFollower>> {
    let mastodon_following = mastodon::get_following(mastodon_user, quiet).await?;

    // Start the process with all users that the bridge account follows on Bluesky that the user's Bluesky account
    // also follows
    let bridgy_did = get_bridgy_did(bluesky).await?;
    let bridgy_followers = get_known_followers(bluesky, &bridgy_did).await?;

    let mut result = Vec::<BridgedFollower>::new();

    // Pass 1: filter accounts ignored in the configuration or already followed on Mastodon.
    // This is the cheapest check, we have all the data to find out right away if we need to process further.
    let to_process = filter_ignored_or_already_following(
        bridgy_followers.values(),
        ignored_accounts,
        &mastodon_following,
        &mut result,
    );

    // Pass 2: relationship checks. Check if the user is really followed by the bridge (It should be the case if
    // get_known_followers returned it) and if the user doesn't block the bridge either directly or via a block
    // list as it would prevent bridging. This remove users that activated bridging but then deactivated it by
    // blocking the bridge.
    let to_process = filter_blocking_bridge(bluesky, &bridgy_did, to_process, &mut result).await?;

    // Pass 3: for all potential new follows check that the user is really bridged by directly querying their
    // profile using the webfinger endpoint of the bridge (acting as an Activity Pub server). This remove users
    // that activated bridging but then deactivated it via the web interface.
    check_webfinger_bridging(to_process, &mut result).await?;

    Ok(result)
}

pub fn write_statuses_to_import_csv<W>(
    csv_writer: &mut csv::Writer<W>,
    statuses: &[BridgedFollower],
) -> csv::Result<()>
where
    W: io::Write,
{
    csv_writer.write_record([
        "Account address",
        "Show boosts",
        "Notify on new posts",
        "Languages",
    ])?;

    for status in statuses {
        if status.status == FollowerStatus::ReadyToFollow {
            let mastodon_handle = crate::utils::bluesky_handle_to_mastodon(&status.handle);
            csv_writer.write_record(&[
                format!("@{mastodon_handle}"),
                "true".to_string(),
                "false".to_string(),
                String::new(),
            ])?;
        }
    }

    Ok(())
}

pub fn statuses_to_import_csv(statuses: &[BridgedFollower]) -> Result<String> {
    let mut csv_writer = csv::Writer::from_writer(vec![]);
    write_statuses_to_import_csv(&mut csv_writer, statuses)?;

    let data = csv_writer.into_inner()?;
    Ok(String::from_utf8(data)?)
}
