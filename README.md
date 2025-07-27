# Bridgy followers

A command-line tool to generate a CSV file containing all the follows from a BlueSky account that also follow
the [Bridgy][Bridgy] account to import them into Mastodon.

[Bridgy]: https://bsky.app/profile/ap.brid.gy

## Usage

```sh
BSKY_PASSWORD=<password> bridgy_followers <username> > new_follows.csv
```

Import the new follows into Mastodon using the `/settings/imports` page (Preferences > Import and Export > Import).