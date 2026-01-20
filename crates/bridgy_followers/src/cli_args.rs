use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub enum Command {
    /// Sync followers from Bluesky to Mastodon (default)
    Sync {
        #[arg(
            default_value = "bridgy_followers.toml",
            help = "Path to configuration file"
        )]
        config: PathBuf,

        #[arg(short, long, help = "Output file (defaults to stdout)")]
        output: Option<PathBuf>,
    },
    /// Clear stored credentials and configuration
    Forget {
        #[arg(
            default_value = "bridgy_followers.toml",
            help = "Path to configuration file"
        )]
        config: PathBuf,
    },
}

#[derive(Parser)]
#[command(name = "bridgy_followers")]
#[command(about = "Find intersection of followers between a user and @ap.brid.gy")]
#[command(args_conflicts_with_subcommands = true)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Command,
}
