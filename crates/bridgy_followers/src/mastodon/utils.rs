use color_eyre::{Result, eyre::WrapErr};
use megalodon::{
    Megalodon, entities::Account, mastodon::Mastodon, megalodon::AccountFollowersInputOptions,
};
use tracing::{info, instrument};

const USER_AGENT: &str = "bridgy_followers";

pub fn create_client(base_url: &str, access_token: Option<String>) -> Result<Mastodon> {
    Mastodon::new(
        base_url.to_string(),
        access_token,
        Some(USER_AGENT.to_string()),
    )
    .wrap_err_with(|| format!("Failed to create Mastodon client at {base_url}"))
}

/// Get all accounts which the given acount is following.
#[instrument(skip(client))]
pub async fn get_account_following(client: &Mastodon, user_id: String) -> Result<Vec<Account>> {
    let mut following = Vec::new();

    let mut response = client
        .get_account_following(
            user_id.clone(),
            Some(&AccountFollowersInputOptions {
                limit: Some(100),
                ..Default::default()
            }),
        )
        .await
        .wrap_err("Failed to get following")?;
    loop {
        let mut accounts = response.json();
        let batch_size = accounts.len();

        if batch_size == 0 {
            break;
        }

        following.append(&mut accounts);

        let next = response.next_uri().wrap_err("Failed to get next page")?;

        if let Some(next) = next {
            response = client
                .get_linked_response(next)
                .await
                .wrap_err("Failed to get continuation of following")?;
        } else {
            break;
        }
    }

    info!("Fetched {} following accounts", following.len());
    Ok(following)
}
