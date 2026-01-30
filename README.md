# Bridgy followers

A command-line tool to sync followers from Bluesky to Mastodon. It finds followers of your Bluesky account that also follow the [Bridgy][Bridgy] account and automatically follows them on Mastodon, or generates a CSV file for manual import.

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

- `sync` - Sync followers from Bluesky to Mastodon (automatically follows new bridged accounts)
- `csv` - Generate a CSV file that can be manually imported into Mastodon
- `forget` - Clear stored credentials and configuration

### Sync command

Automatically follows new bridged accounts on Mastodon:

```sh
bridgy_followers sync [config_file]
```

- `[config_file]` - Path to configuration file (defaults to `bridgy_followers.toml`)

### CSV command

Generate a CSV file for manual import into Mastodon:

```sh
bridgy_followers csv [config_file] [-o OUTPUT]
```

- `[config_file]` - Path to configuration file (defaults to `bridgy_followers.toml`)
- `-o, --output <FILE>` - Write output to a file instead of stdout

### Examples

Automatically follow new bridged accounts:

```sh
bridgy_followers sync
```

Generate CSV for manual import:

```sh
bridgy_followers csv -o followers.csv
```

Use a custom configuration file:

```sh
bridgy_followers sync my_config.toml
```

Clear stored credentials:

```sh
bridgy_followers forget
```

To manually import a CSV file into Mastodon, use the `/settings/imports` page (Preferences > Import and Export > Import).
