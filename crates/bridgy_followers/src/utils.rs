use atrium_api::types::string::Handle;

pub fn bluesky_handle_to_mastodon(handle: &Handle) -> String {
    format!("{}@bsky.brid.gy", handle.as_str())
}

pub fn color_bool(value: bool) -> String {
    if value {
        "\x1b[32mtrue\x1b[0m".to_string() // Green
    } else {
        "\x1b[31mfalse\x1b[0m".to_string() // Red
    }
}
