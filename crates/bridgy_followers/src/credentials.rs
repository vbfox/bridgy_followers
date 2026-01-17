use keyring::{Credential, CredentialBuilder};

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
        let _ = credential.delete_credential();
    }

    // Delete Bluesky credential if username is known
    if let Some(username) = bluesky_username
        && let Ok(credential) = get_bluesky_password(credential_builder, username)
    {
        let _ = credential.delete_credential();
    }
}
