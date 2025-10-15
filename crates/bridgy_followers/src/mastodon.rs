use color_eyre::{Result, eyre::eyre};
use dialoguer::Input;
use megalodon::megalodon::AppInputOptions;
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
    let app_data = megalodon::generator(
        megalodon::SNS::Mastodon,
        server_url.clone(),
        None,
        Some(String::from("bridgy_followers")),
    )
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

    println!("\nPlease open this URL in your browser:\n{}\n", authorize_url);

    let auth_code: String = Input::new()
        .with_prompt("Enter the authorization code")
        .interact_text()?;

    println!("Getting access token...");
    let token_data = megalodon::generator(
        megalodon::SNS::Mastodon,
        server_url.clone(),
        None,
        Some(String::from("bridgy_followers")),
    )
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
    let client = megalodon::generator(
        megalodon::SNS::Mastodon,
        config.server.clone(),
        Some(config.access_token.clone()),
        Some(String::from("bridgy_followers")),
    );

    println!("Fetching current user...");
    let account = client.verify_account_credentials().await
        .map_err(|e| eyre!("Failed to verify credentials: {}", e))?;
    let user_id = account.json().id;

    println!("Fetching following list from Mastodon...");
    let mut following = HashSet::new();
    let mut max_id: Option<String> = None;
    let mut total_fetched = 0;

    loop {
        println!("Fetching next batch of following {max_id:?}...");
        let response = client.get_account_following(
            user_id.clone(),
            Some(&megalodon::megalodon::AccountFollowersInputOptions {
                max_id: max_id.clone(),

                limit: Some(5),
                ..Default::default()
            }),
        ).await
        .map_err(|e| eyre!("Failed to get following: {}", e))?;

        let accounts = response.json();
        let batch_size = accounts.len();

        println!("{:?}", response.header);

        if batch_size == 0 {
            break;
        }

        println!("{:?}", accounts.iter().map(|a|a.id.clone()).collect::<Vec<_>>());

        total_fetched += batch_size;
        println!("Fetched {} accounts (total: {})...", batch_size, total_fetched);

        let new_max_id = accounts.last().map(|a| a.id.clone());

        println!("{:?}", accounts.last());

        max_id = new_max_id;

        for account in accounts {
            following.insert(account.acct.to_lowercase());
        }
    }

    println!("Found {} accounts you're following on Mastodon", following.len());

    Ok(following)
}
