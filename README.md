# Bridgy followers

A command-line tool to sync followers from Bluesky to Mastodon. It finds followers of your Bluesky account that also follow the [Bridgy][Bridgy] account and generates a CSV file to import them into Mastodon.

[Bridgy]: https://bsky.app/profile/ap.brid.gy

## Configuration

Create a `bridgy_followers.toml` configuration file:

```toml
bluesky_username = "your.handle.bsky.social"
mastodon_server = "mastodon.social"
ignored_accounts = ["user1.bsky.social", "user2.bsky.social"]
```

The `ignored_accounts` list is optional and allows you to exclude specific accounts from the output.

Credentials (Bluesky app password and Mastodon access token) are stored securely in your system keyring and will be prompted for on first run.

## Usage

```sh
bridgy_followers <COMMAND>
```

### Commands

- `sync` - Sync followers from Bluesky to Mastodon (default)
- `forget` - Clear stored credentials and configuration

### Sync command

```sh
bridgy_followers sync [config_file] [-o OUTPUT]
```

- `[config_file]` - Path to configuration file (defaults to `bridgy_followers.toml`)
- `-o, --output <FILE>` - Write output to a file instead of stdout

### Examples

Output to stdout:

```sh
bridgy_followers sync
```

Output to a file with a custom configuration file:

```sh
bridgy_followers sync my_config.toml -o followers.csv
```

Clear stored credentials:

```sh
bridgy_followers forget
```

Import the new follows into Mastodon using the `/settings/imports` page (Preferences > Import and Export > Import).
