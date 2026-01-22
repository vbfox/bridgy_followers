use atrium_api::types::string::Handle;

pub fn bluesky_handle_to_mastodon(handle: &Handle) -> String {
    format!("{}@bsky.brid.gy", handle.as_str())
}
