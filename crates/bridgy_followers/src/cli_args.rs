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

        /// Increase verbosity level (-v, -vv, -vvv)
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
    },
    /// Clear stored credentials and configuration
    Forget {
        #[arg(
            default_value = "bridgy_followers.toml",
            help = "Path to configuration file"
        )]
        config: PathBuf,

        /// Increase verbosity level (-v, -vv, -vvv)
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
    },
}

impl Command {
    pub fn verbose(&self) -> u8 {
        match self {
            Command::Sync { verbose, .. } => *verbose,
            Command::Forget { verbose, .. } => *verbose,
        }
    }
}

#[derive(Parser)]
#[command(name = "bridgy_followers")]
#[command(about = "Find intersection of followers between a user and @ap.brid.gy")]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Command,
}
