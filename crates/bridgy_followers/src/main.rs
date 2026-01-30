#![allow(clippy::borrowed_box, reason = "Trigger on &Box<dyn Trait> parameters")]

use crate::cli_args::{CliArgs, Command};
use crate::commands::{config_command, csv_command, forget_command, sync_command};
use clap::Parser;
use color_eyre::Result;

mod bluesky;
mod cli_args;
mod commands;
mod config;
mod credentials;
mod follower_status;
mod mastodon;
mod tracing;
mod utils;
mod webfinger;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let cli = CliArgs::parse();
    tracing::init_tracing(cli.command.verbose());

    match cli.command {
        Command::Sync { config, .. } => sync_command(config, None).await,
        Command::Csv { config, output, .. } => csv_command(config, output).await,
        Command::Forget { config, .. } => forget_command(&config),
        Command::Config { .. } => config_command(),
    }
}
