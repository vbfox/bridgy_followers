use keyring::{Credential, CredentialBuilder};
use tracing::{debug, info};

/// Get the Keyring credential for storing/retrieving the Bluesky password
pub fn get_bluesky_password(
    credential_builder: &Box<CredentialBuilder>,
    user_name: &str,
) -> keyring::Result<Box<Credential>> {
    credential_builder.build(None, "bridgy_followers", &format!("bluesky_{user_name}"))
}

/// Get the Keyring credential for storing/retrieving the Mastodon access token
pub fn get_mastodon_access_token(
    credential_builder: &Box<CredentialBuilder>,
    server: &str,
) -> keyring::Result<Box<Credential>> {
    credential_builder.build(
        None,
        "bridgy_followers",
        &format!("mastodon_access_token_{server}"),
    )
}

/// Delete stored credentials from keyring
pub fn delete_credentials(
    credential_builder: &Box<CredentialBuilder>,
    mastodon_server: Option<&str>,
    bluesky_username: Option<&str>,
) {
    // Delete Mastodon credential if server is known
    if let Some(server) = mastodon_server
        && let Ok(credential) = get_mastodon_access_token(credential_builder, server)
    {
        match credential.delete_credential() {
            Ok(_) => info!("Deleted Mastodon credentials for server '{server}'"),
            Err(e) => {
                if let keyring::Error::NoEntry = e {
                    debug!("No Mastodon credentials found for server '{server}'");
                }
                eprintln!("Failed to delete Mastodon credentials for server '{server}': {e}");
            }
        }
    }

    // Delete Bluesky credential if username is known
    if let Some(username) = bluesky_username
        && let Ok(credential) = get_bluesky_password(credential_builder, username)
    {
        match credential.delete_credential() {
            Ok(_) => info!("Deleted Bluesky credentials for username '{username}'"),
            Err(e) => {
                if let keyring::Error::NoEntry = e {
                    debug!("No Bluesky credentials found for username '{username}'");
                }
                eprintln!("Failed to delete Bluesky credentials for username '{username}': {e}");
            }
        }
    }
}
