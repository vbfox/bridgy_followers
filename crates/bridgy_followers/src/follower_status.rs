use crate::{
    bluesky::{BlueskyAgent, get_bridgy_did, get_known_followers, get_relationships},
    mastodon,
    utils::{BRIDGY_ACTIVITY_PUB_URL, bluesky_handle_to_mastodon},
    webfinger,
};
use atrium_api::types::string::Handle;
use color_eyre::Result;
use ipld_core::ipld::Ipld;
use megalodon::mastodon::Mastodon;
use std::collections::BTreeMap;
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

pub async fn get_follower_statuses(
    mastodon_user: &Mastodon,
    bluesky: &BlueskyAgent,
    ignored_accounts: &[String],
) -> Result<Vec<BridgedFollower>> {
    let mastodon_following = mastodon::get_following(&mastodon_user).await?;

    let bridgy_did = get_bridgy_did(&bluesky).await?;
    let bridgy_followers = get_known_followers(&bluesky, &bridgy_did).await?;
    let to_process = bridgy_followers.values();

    let mut result = Vec::<BridgedFollower>::new();

    // ----------------------------------------------------------------------
    // Pass 1: ignored and already-followed
    // This is the cheapest check, we have all the data to find out right away if we need to process further
    let to_process: Vec<_> = to_process
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
        .collect();

    // ----------------------------------------------------------------------
    // Pass 2: relationship checks
    // We check if the bridge follow the users on Bluesky, this check is cheap but incomplete as in some cases where
    // the user bridged at some point but no longer it's not good enough.

    let relationships = get_relationships(
        &bluesky,
        bridgy_did.clone().into(),
        to_process.iter().map(|f| f.did.clone().into()),
    )
    .await?;

    let to_process: Vec<_> = to_process
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
                    // Blocks are a Recent adition to the Lexicon, not yet in atrium
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

                    // TODO: Find out why there are weird cases;
                    // - '@thornbulle.bsky.social' doesn't report following the bridge but is bridged
                    // - '@terribletoybox.com@bsky.brid.gy' follows both way but is not bridged
                    // - '@pwnallthethings.bsky.social@bsky.brid.gy' same

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
        .collect();

    // ----------------------------------------------------------------------
    // Pass 3: for all potential new follows check that the user is really bridged by directly querying their profile
    // using the webfinger endpoint of the bridge (acting as an Activity Pub server)

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

    Ok(result)
}

pub fn write_statuses_to_import_csv<W>(
    csv_writer: &mut csv::Writer<W>,
    statuses: &[BridgedFollower],
) -> csv::Result<()>
where
    W: std::io::Write,
{
    csv_writer.write_record(&[
        "Account address",
        "Show boosts",
        "Notify on new posts",
        "Languages",
    ])?;

    for status in statuses {
        match &status.status {
            FollowerStatus::ReadyToFollow => {
                let mastodon_handle = crate::utils::bluesky_handle_to_mastodon(&status.handle);
                csv_writer.write_record(&[
                    format!("@{mastodon_handle}"),
                    "true".to_string(),
                    "false".to_string(),
                    "".to_string(),
                ])?;
            }
            _ => {}
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
