use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use clap::{Parser, Subcommand};

type DynError = Box<dyn std::error::Error>;

// We currently build megalodon using a path.crates.io dependency and it generates dead code warnings.
const RUSTFLAGS: &str = "-D warnings -A dead_code";

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
    /// Run cargo fmt for the whole workspace
    Fmt {
        /// Check formatting without applying any changes
        #[arg(long)]
        check: bool,
    },
    /// Run the bridgy_followers binary
    #[command(disable_help_flag = true, disable_version_flag = true)]
    Run {
        /// Arguments passed through to the bridgy_followers binary
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.task {
        Task::Build => build(),
        Task::Clippy => clippy(),
        Task::Fmt { check } => fmt(check),
        Task::Run { args } => run(args),
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
        .env("RUSTFLAGS", RUSTFLAGS)
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
        .env("RUSTFLAGS", RUSTFLAGS)
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

fn fmt(check: bool) -> Result<(), DynError> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut args = vec!["fmt", "--all"];
    if check {
        args.extend(["--", "--check"]);
    }

    let status = Command::new(cargo)
        .current_dir(project_root())
        .args(args)
        .status()?;

    if !status.success() {
        Err("cargo fmt failed")?;
    }
    Ok(())
}

fn run(args: Vec<String>) -> Result<(), DynError> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    let mut cargo_args = vec![
        "run".to_string(),
        "--package".to_string(),
        "bridgy_followers".to_string(),
    ];
    if !args.is_empty() {
        cargo_args.push("--".to_string());
        cargo_args.extend(args);
    }

    let status = Command::new(cargo)
        .current_dir(project_root())
        .env("RUSTFLAGS", RUSTFLAGS)
        .args(cargo_args)
        .status()?;

    if !status.success() {
        Err("cargo run failed")?;
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
