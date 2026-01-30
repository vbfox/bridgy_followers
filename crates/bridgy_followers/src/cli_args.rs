use std::path::PathBuf;

use clap::Parser;

use crate::config;

fn default_config_path() -> PathBuf {
    config::default_config_path().unwrap_or_else(|_| PathBuf::from("bridgy_followers.toml"))
}

#[derive(Parser)]
pub enum Command {
    /// Sync followers from Bluesky to Mastodon (follows new bridged accounts automatically)
    Sync {
        /// Path to configuration file
        #[arg(default_value_os_t = default_config_path())]
        config: PathBuf,

        /// Increase verbosity level.
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
    },
    /// Generate a CSV that can be imported into mastodon UI
    Csv {
        /// Path to configuration file
        #[arg(default_value_os_t = default_config_path())]
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
        #[arg(default_value_os_t = default_config_path())]
        config: PathBuf,

        /// Increase verbosity level.
        #[arg(short, long, action = clap::ArgAction::Count)]
        verbose: u8,
    },
    // Get the default config path
    Config {
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
            | Command::Csv { verbose, .. }
            | Command::Config { verbose, .. } => *verbose,
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
