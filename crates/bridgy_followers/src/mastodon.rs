use color_eyre::{Result, eyre::eyre};
use dialoguer::Input;
use megalodon::{
    Megalodon,
    megalodon::{AccountFollowersInputOptions, AppInputOptions},
};
use std::collections::HashSet;

use crate::config::MastodonConfig;

pub async fn authenticate() -> Result<MastodonConfig> {
    let server: String = Input::new()
        .with_prompt("Mastodon server (e.g., mastodon.social)")
        .interact_text()?;

    let server_url = if server.starts_with("http") {
        server.clone()
    } else {
        format!("https://{}", server)
    };

    println!("Registering application...");
    let client = megalodon::generator(
        megalodon::SNS::Mastodon,
        server_url.clone(),
        None,
        Some(String::from("bridgy_followers")),
    )?;
    let app_data = client
        .register_app(
            String::from("Bridgy Followers"),
            &AppInputOptions {
                scopes: Some(vec![
                    String::from("read:accounts"),
                    String::from("read:follows"),
                ]),
                redirect_uris: Some(String::from("urn:ietf:wg:oauth:2.0:oob")),
                website: None,
            },
        )
        .await
        .map_err(|e| eyre!("Failed to register app: {}", e))?;

    let authorize_url = app_data.url.clone().unwrap_or_else(|| {
        format!(
            "{}/oauth/authorize?client_id={}&redirect_uri=urn:ietf:wg:oauth:2.0:oob&response_type=code&scope=read:accounts+read:follows",
            server_url, app_data.client_id
        )
    });

    println!(
        "\nPlease open this URL in your browser:\n{}\n",
        authorize_url
    );

    let auth_code: String = Input::new()
        .with_prompt("Enter the authorization code")
        .interact_text()?;

    println!("Getting access token...");
    let token_data = client
        .fetch_access_token(
            app_data.client_id,
            app_data.client_secret,
            auth_code,
            String::from("urn:ietf:wg:oauth:2.0:oob"),
        )
        .await
        .map_err(|e| eyre!("Failed to fetch access token: {}", e))?;

    let config = MastodonConfig {
        server: server_url,
        access_token: token_data.access_token,
    };

    println!("✓ Authentication successful!");

    Ok(config)
}

pub async fn get_following(config: &MastodonConfig) -> Result<HashSet<String>> {
    let client = megalodon::mastodon::Mastodon::new(
        config.server.clone(),
        Some(config.access_token.clone()),
        Some(String::from("bridgy_followers")),
    )?;

    println!("Fetching current user...");
    let account = client
        .verify_account_credentials()
        .await
        .map_err(|e| eyre!("Failed to verify credentials: {}", e))?;
    let user_id = account.json().id;

    println!("Fetching following list from Mastodon...");
    let mut following = HashSet::new();

    let mut response = client
        .get_account_following(
            user_id.clone(),
            Some(&AccountFollowersInputOptions {
                limit: Some(100),
                ..Default::default()
            }),
        )
        .await
        .map_err(|e| eyre!("Failed to get following: {}", e))?;
    loop {
        let accounts = response.json();
        let batch_size = accounts.len();

        if batch_size == 0 {
            break;
        }

        for account in accounts {
            following.insert(account.acct.to_lowercase());
        }

        let next = response
            .next_uri()
            .map_err(|e| eyre!("Failed to get next page: {}", e))?;

        if let Some(next) = next {
            response = client
                .get_linked_response(next)
                .await
                .map_err(|e| eyre!("Failed to get continuation of following: {}", e))?;
        } else {
            break;
        }
    }

    println!(
        "Found {} accounts you're following on Mastodon",
        following.len()
    );

    Ok(following)
}
