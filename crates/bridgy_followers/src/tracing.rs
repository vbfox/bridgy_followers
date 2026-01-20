use tracing_subscriber::{EnvFilter, fmt, prelude::*};

pub fn init_tracing(verbose: u8) {
    // Determine the filter level based on verbosity count
    let filter = match verbose {
        0 => return, // No tracing
        1 => "info",
        2 => "debug",
        _ => "trace", // 3 or more
    };

    // Build the EnvFilter, respecting RUST_LOG if set
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(filter))
        .unwrap();

    // Initialize the tracing subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer())
        .init();
}
