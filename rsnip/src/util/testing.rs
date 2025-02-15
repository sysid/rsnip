// src/util/testing.rs

use std::env;
use anyhow::Result;
use tracing::{debug, info};
use tracing_subscriber::{
    filter::filter_fn,
    fmt::{self, format::FmtSpan},
    prelude::*,
    EnvFilter,
};

// Common test environment variables
pub const TEST_ENV_VARS: &[&str] = &["RUST_LOG", "NO_CLEANUP"];

pub fn init_test_setup() -> Result<()> {
    // Set up logging first
    setup_test_logging();

    info!("Test Setup complete");
    Ok(())
}

fn setup_test_logging() {
    debug!("INIT: Attempting logger init from testing.rs");
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "trace");
    }

    // Create a filter for noisy modules
    let noisy_modules = ["skim", "html5ever", "reqwest", "mio"];
    let module_filter = filter_fn(move |metadata| {
        !noisy_modules
            .iter()
            .any(|name| metadata.target().starts_with(name))
    });

    // Set up the subscriber with environment filter
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

    // Build and set the subscriber
    let subscriber = tracing_subscriber::registry().with(
        fmt::layer()
            .with_writer(std::io::stderr)
            .with_target(true)
            .with_thread_names(false)
            .with_span_events(FmtSpan::CLOSE)
            .with_filter(module_filter)
            .with_filter(env_filter),
    );

    // Only set if we haven't already set a global subscriber
    if tracing::dispatcher::has_been_set() {
        debug!("Tracing subscriber already set");
    } else {
        subscriber.try_init().unwrap_or_else(|e| {
            eprintln!("Error: Failed to set up logging: {}", e);
        });
    }
}

pub fn print_active_env_vars() {
    for var in TEST_ENV_VARS {
        if let Ok(value) = env::var(var) {
            println!("{var}={value}");
        } else {
            println!("{var} is not set");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    

    #[ctor::ctor]
    fn init() {
        init_test_setup().expect("Failed to initialize test setup");
    }
}
