use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use clap::{Parser, Subcommand};

type DynError = Box<dyn std::error::Error>;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    task: Task,
}

#[derive(Subcommand)]
enum Task {
    /// Run cargo build for the whole workspace
    Build,
    /// Run clippy the same way CI does, including its RUSTFLAGS
    Clippy,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.task {
        Task::Build => build(),
        Task::Clippy => clippy(),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn build() -> Result<(), DynError> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .current_dir(project_root())
        .args(["build", "--workspace", "--exclude", "xtask"])
        .status()?;

    if !status.success() {
        Err("cargo build failed")?;
    }
    Ok(())
}

fn clippy() -> Result<(), DynError> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let status = Command::new(cargo)
        .current_dir(project_root())
        // We currently build megalodon using path.crates.io dependency and it generate dead code warnings.
        .env("RUSTFLAGS", "-D warnings -A dead_code")
        .args([
            "clippy",
            "--all-targets",
            "--all-features",
            "--tests",
            "--benches",
            "--",
            "-Dclippy::all",
            "-Dclippy::pedantic",
        ])
        .status()?;

    if !status.success() {
        Err("cargo clippy failed")?;
    }
    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}
