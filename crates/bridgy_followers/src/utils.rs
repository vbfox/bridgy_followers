use atrium_api::types::string::Handle;

pub const BRIDGY_ACTIVITY_PUB_URL: &str = "https://fed.brid.gy";

const BRIDGY_MASTODON_DOMAIN: &str = "bsky.brid.gy";

pub fn bluesky_handle_to_mastodon(handle: &Handle) -> String {
    format!("{}@{}", handle.as_str(), BRIDGY_MASTODON_DOMAIN).to_lowercase()
}
