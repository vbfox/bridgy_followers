use color_eyre::{
    Result,
    eyre::{WrapErr, eyre},
    owo_colors::OwoColorize,
};
use dialoguer::{Input, Password, theme::ColorfulTheme};

use keyring::CredentialBuilder;
use megalodon::{Megalodon, mastodon::Mastodon, megalodon::AppInputOptions};
use std::collections::HashSet;

use super::utils::create_client;
use crate::{
    config::{Config, ConfigData},
    credentials,
    mastodon::utils::get_account_following,
};

/// Prompt the user for the Mastodon server if not already set in config
fn get_server(config: &mut Config) -> Result<String> {
    if let Some(server) = config.mastodon_server() {
        Ok(server.to_string())
    } else {
        let input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Mastodon server (e.g., mastodon.social or https://hachyderm.io)")
            .interact_text()?;

        let server = if input.starts_with("http") {
            input.clone()
        } else {
            format!("https://{input}")
        };
        config.mutate(|data| ConfigData {
            mastodon_server: Some(server.clone()),
            ..data
        })?;
        Ok(server)
    }
}

async fn register_application(server_url: String) -> Result<String> {
    println!("Registering application...");
    let client = create_client(&server_url, None)?;
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
        .wrap_err("Failed to register app")?;

    let authorize_url = app_data.url.clone().unwrap_or_else(|| {
        format!(
            "{}/oauth/authorize?client_id={}&redirect_uri=urn:ietf:wg:oauth:2.0:oob&response_type=code&scope=read:accounts+read:follows",
            server_url, app_data.client_id
        )
    });

    println!("\nPlease open this URL in your browser:\n{authorize_url}\n");

    let auth_code: String = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter the authorization code")
        .interact()?;

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

    let access_token = token_data.access_token;

    println!("✓ Authentication successful!");

    Ok(access_token)
}

pub async fn authenticate(
    credential_builder: &Box<CredentialBuilder>,
    config: &mut Config,
) -> Result<Mastodon> {
    let server_url = get_server(config)?;

    let credentials = credentials::get_mastodon_access_token(credential_builder, &server_url)?;

    let access_token = if let Ok(token) = credentials.get_password() {
        token
    } else {
        let token = register_application(server_url.clone()).await?;
        credentials.set_password(&token)?;
        token
    };

    let client = create_client(&server_url, Some(access_token))?;

    // TODO: We should use verify_account_credentials here to ensure the token is valid and prompt for
    // re-authentication or server-change if not.

    Ok(client)
}

pub async fn get_following(client: &Mastodon) -> Result<HashSet<String>> {
    println!("Fetching current user...");

    let account_response = client
        .verify_account_credentials()
        .await
        .map_err(|e| eyre!("Failed to verify credentials: {}", e))?;
    let account = account_response.json();
    println!(
        "Fetching {} following list from Mastodon...",
        format!("@{}", &account.acct).blue()
    );

    let user_id = account.id;
    let following = get_account_following(client, user_id).await?;
    println!(
        "Found {} accounts you're following on Mastodon",
        following.len().yellow()
    );

    let following: HashSet<String> = following.into_iter().map(|account| account.acct).collect();

    Ok(following)
}
