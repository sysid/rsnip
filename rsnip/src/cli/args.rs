use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Debug, Parser)]
#[command(name = "rsnip", version = "0.1.0")]
pub struct Cli {
    /// Enable debug logging. Multiple flags (-d, -dd, -ddd) increase verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// Generate shell completion scripts
    #[arg(long = "generate", value_enum)]
    pub generator: Option<Shell>,

    /// Display version and configuration information
    #[arg(long = "info")]
    pub info: bool,

    #[command(subcommand)]
    pub(crate) command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Find completions with optional interactive selection
    Complete {
        /// Type of completion e.g. "mytype"
        #[arg(long)]
        ctype: String,
        /// The partial input to match on
        #[arg(long)]
        input: Option<String>,
        /// Use interactive selection with fzf
        #[arg(short, long)]
        interactive: bool,
    },
    /// Example command using completion result
    Xxx {
        /// Type of completion e.g. "mytype"
        #[arg(short, long)]
        ctype: String,
        /// The text to look up from completion
        #[arg(long)]
        input: String,
    },
    /// Copy text to clipboard
    Copy {
        /// Type of completion e.g. "mytype"
        #[arg(short, long)]
        ctype: String,
        /// The text to copy
        #[arg(long)]
        input: String,
    },
}