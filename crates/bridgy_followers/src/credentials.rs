use keyring::{Credential, CredentialBuilder};

/// Get the Keyring credential for storing/retrieving the Bluesky password
pub fn get_bluesky_password(credential_builder: &Box<CredentialBuilder>, user_name: &str) -> keyring::Result<Box<Credential>> {
    credential_builder.build(
        None,
        "bridgy_followers",
        &format!("bluesky_{user_name}"),
    )
}

/// Get the Keyring credential for storing/retrieving the Mastodon access token
pub fn get_mastodon_access_token(credential_builder: &Box<CredentialBuilder>, server: &str) -> keyring::Result<Box<Credential>> {
    credential_builder.build(
        None,
        "bridgy_followers",
        &format!("mastodon_access_token_{server}"),
    )
}
