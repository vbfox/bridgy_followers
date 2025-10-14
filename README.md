# Bridgy followers

A command-line tool to generate a CSV file containing all the follows from a BlueSky account that also follow
the [Bridgy][Bridgy] account to import them into Mastodon.

[Bridgy]: https://bsky.app/profile/ap.brid.gy

## Configuration

Create a `bridgy_followers.toml` configuration file:

```toml
username = "your.handle"
password = "your-password"
ignored_accounts = ["user1.bsky.social", "user2.bsky.social"]
```

The `ignored_accounts` list is optional and allows you to exclude specific accounts from the output.

## Usage

```sh
bridgy_followers [config_file]
```

If no config file is specified, it defaults to `bridgy_followers.toml`.

Example:
```sh
bridgy_followers > new_follows.csv
# or with custom config path
bridgy_followers my_config.toml > new_follows.csv
```

Import the new follows into Mastodon using the `/settings/imports` page (Preferences > Import and Export > Import).
