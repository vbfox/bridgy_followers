# Bridgy followers

A command-line tool to sync followers from Bluesky to Mastodon. It finds followers of your Bluesky account that also follow the [Bridgy][Bridgy] account and automatically follows them on Mastodon, or generates a CSV file for manual import.

[Bridgy]: https://bsky.app/profile/ap.brid.gy

## Configuration

The configuration file is stored in a platform-dependent location:

- **Linux**: `~/.config/bridgy_followers/config.toml` (follows XDG Base Directory specification)
- **Windows**: `%APPDATA%\vbfox\bridgy_followers\config\config.toml` (typically `C:\Users\<User>\AppData\Roaming\vbfox\bridgy_followers\config\`)
- **macOS**: `~/Library/Application Support/net.vbfox.bridgy_followers/config.toml`

The configuration file will be created automatically on first run. You can also specify a custom configuration file path using the command-line arguments.

### Configuration format

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
- `ignores` - Manage ignored accounts interactively with a TUI
- `forget` - Clear stored credentials and configuration
- `config` - Show the default configuration file path

### Sync command

Automatically follows new bridged accounts on Mastodon:

```sh
bridgy_followers sync [config_file]
```

- `[config_file]` - Optional path to custom configuration file (defaults to platform-dependent location)

### CSV command

Generate a CSV file for manual import into Mastodon:

```sh
bridgy_followers csv [config_file] [-o OUTPUT]
```

- `[config_file]` - Optional path to custom configuration file (defaults to platform-dependent location)
- `-o, --output <FILE>` - Write output to a file instead of stdout

### Ignores command

Interactively manage ignored accounts with a TUI:

```sh
bridgy_followers ignores [config_file]
```

- `[config_file]` - Optional path to custom configuration file (defaults to platform-dependent location)

This command displays a list of all currently ignored accounts. Use the space bar to select accounts to remove from the ignore list, then press Enter to confirm.

### Examples

Automatically follow new bridged accounts:

```sh
bridgy_followers sync
```

Generate CSV for manual import:

```sh
bridgy_followers csv -o followers.csv
```

Manage ignored accounts interactively:

```sh
bridgy_followers ignores
```

Use a custom configuration file:

```sh
bridgy_followers sync my_config.toml
```

Clear stored credentials:

```sh
bridgy_followers forget
```

Show the default config file location:

```sh
bridgy_followers config
```

To manually import a CSV file into Mastodon, use the `/settings/imports` page (Preferences > Import and Export > Import).
