use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
pub enum Command {
    /// Sync followers from Bluesky to Mastodon
    Sync {
        /// Path to configuration file
        #[arg(default_value = "bridgy_followers.toml")]
        config: PathBuf,

        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Increase verbosity level.
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
    },
    /// Generate a CSV that can be imported into mastodon UI
    Csv {
        /// Path to configuration file
        #[arg(default_value = "bridgy_followers.toml")]
        config: PathBuf,

        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Increase verbosity level.
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
    },
    /// Clear stored credentials and configuration
    Forget {
        /// Path to configuration file
        #[arg(default_value = "bridgy_followers.toml")]
        config: PathBuf,

        /// Increase verbosity level.
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
    },
}

impl Command {
    pub fn verbose(&self) -> u8 {
        match self {
            Command::Sync { verbose, .. }
            | Command::Forget { verbose, .. }
            | Command::Csv { verbose, .. } => *verbose,
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
