use clap::Parser;
use crossterm::style::Stylize;
use rsnip::cli::args::Cli;
use rsnip::cli::commands::execute_command;
use rsnip::complete::generate_completion_script;
use rsnip::config::{Settings, SnippetTypeConfig};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, Layer};

fn main() {
    let cli = Cli::parse();

    setup_logging(cli.debug);

    // Handle generate-config before any other operations
    if cli.generate_config {
        println!("{}", include_str!("default_config.toml"));
        return;
    }

    // Initialize configuration
    let config = match Settings::new() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{}", format!("Error loading configuration: {}", e).red());
            std::process::exit(1);
        }
    };

    if let Some(shell) = cli.generator {
        if let Err(e) = generate_completion_script(shell, std::io::stdout(), &config) {
            eprintln!(
                "{}",
                format!("Error generating completion script: {}", e).red()
            );
            std::process::exit(1);
        }
        return;
    }

    if cli.info {
        use clap::CommandFactory; // Trait which returns the current command
        if let Some(a) = Cli::command().get_author() {
            println!("AUTHOR: {}", a)
        }
        if let Some(v) = Cli::command().get_version() {
            println!("VERSION: {}", v);
        }
        // Print config file location
        match &config.active_config_path {
            Some(path) => println!("CONFIG: {}", path.display()),
            None => println!("CONFIG: Using default embedded configuration"),
        }
        // Print available snippet types
        println!("\nAvailable snippet types:");
        for (name, cfg) in &config.snippet_types {
            match cfg {
                SnippetTypeConfig::Concrete { description, .. }
                | SnippetTypeConfig::Combined { description, .. } => {
                    if let Some(desc) = description {
                        println!("  {}: {}", name, desc);
                    } else {
                        println!("  {}", name);
                    }
                }
            }
        }
    }

    if let Err(e) = execute_command(&cli, &config) {
        eprintln!("{}", format!("Error: {}", e).red());
        std::process::exit(1);
    }
}

fn setup_logging(verbosity: u8) {
    tracing::debug!("INIT: Attempting logger init from main.rs");

    let filter = match verbosity {
        0 => LevelFilter::WARN,
        1 => LevelFilter::INFO,
        2 => LevelFilter::DEBUG,
        3 => LevelFilter::TRACE,
        _ => {
            eprintln!("Don't be crazy, max is -d -d -d");
            LevelFilter::TRACE
        }
    };

    // Create a noisy module filter (Gotcha: empty matches all!)
    let noisy_modules = ["x"];
    let module_filter = filter_fn(move |metadata| {
        !noisy_modules
            .iter()
            .any(|name| metadata.target().starts_with(name))
    });

    // Create a subscriber with formatted output directed to stderr
    let fmt_layer = fmt::layer()
        .with_writer(std::io::stderr) // Set writer first
        .with_target(true)
        .with_thread_names(false)
        .with_span_events(FmtSpan::ENTER)
        .with_span_events(FmtSpan::CLOSE);

    // Apply filters to the layer
    let filtered_layer = fmt_layer.with_filter(filter).with_filter(module_filter);

    tracing_subscriber::registry().with(filtered_layer).init();

    // Log initial debug level
    match filter {
        LevelFilter::INFO => tracing::info!("Debug mode: info"),
        LevelFilter::DEBUG => tracing::debug!("Debug mode: debug"),
        LevelFilter::TRACE => tracing::debug!("Debug mode: trace"),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tracing::info;

    // https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html#testing
    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
        info!("CLI verified");
    }
}
