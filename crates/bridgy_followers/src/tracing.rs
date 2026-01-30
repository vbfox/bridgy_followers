use color_eyre::owo_colors::OwoColorize;
use tracing_subscriber::{
    EnvFilter,
    fmt::{self, format::debug_fn},
    prelude::*,
};

pub fn init_tracing(verbose: u8) {
    // If RUST_LOG is set, respect it and ignore verbosity level
    if std::env::var("RUST_LOG").is_ok() {
        let env_filter = EnvFilter::try_from_default_env().unwrap();
        tracing_subscriber::registry()
            .with(env_filter)
            .with(fmt::layer())
            .init();
        return;
    }

    // Determine the filter level based on verbosity count
    let filter = match verbose {
        0 => return, // No tracing
        1 => "bridgy_followers=info",
        2 => "bridgy_followers=debug,error",
        3 => "bridgy_followers=debug,info",
        4 => "bridgy_followers=trace,debug",
        _ => "trace",
    };

    // Build the EnvFilter
    let env_filter = EnvFilter::try_new(filter).unwrap();

    let fmt_layer = match verbose {
        // `-v` show only the message in a dimmed format
        1 => fmt::layer()
            .with_level(false)
            .without_time()
            .with_target(false)
            .map_fmt_fields(|f| f.display_messages())
            .fmt_fields(debug_fn(|writer, field, value| {
                if field.name() == "message" {
                    write!(writer, "{}", format!("{:?}", value).dimmed())
                } else {
                    Ok(())
                }
            }))
            .boxed(),
        // `-vv` show all fields but still no timestamps
        2 => fmt::layer().without_time().boxed(),
        // `-vvv` and higher show full logs with timestamps
        _ => fmt::layer().boxed(),
    };

    // Initialize the tracing subscriber
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .init();
}

/// Conditionally writes to either the tracing info! macro or println! based on a boolean.
///
/// # Usage
/// ```
/// write_or_info!(quiet, "Hello, world!");
/// write_or_info!(quiet, "Value: {}", 42);
/// write_or_info!(quiet, field1 = value1, field2 = value2, "Message");
/// ```
#[macro_export]
macro_rules! println_or_info {
    ($quiet:expr, $($arg:tt)*) => {
        if $quiet {
            tracing::info!($($arg)*);
        } else {
            println!($($arg)*);
        }
    };
}
