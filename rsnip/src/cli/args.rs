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
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List available snippet types
    Types,
    /// Edit snippet file in system editor
    Edit {
        /// Type of completion
        #[arg(long)]
        ctype: Option<String>,
    },
    /// Find completions with optional interactive selection
    Complete {
        /// Type of completion
        #[arg(long)]
        ctype: Option<String>,
        /// The partial input to match on
        #[arg(long)]
        input: Option<String>,
        /// Use interactive selection with fzf
        #[arg(short, long)]
        interactive: bool,
    },
    /// Copy text to clipboard
    Copy {
        /// Type of completion
        #[arg(long)]
        ctype: Option<String>,
        /// The text to copy
        #[arg(long)]
        input: String,
    },
    Xxx {
        /// Type of completion
        #[arg(long)]
        ctype: Option<String>,
        /// The text to look up from completion
        #[arg(long)]
        input: String,
    },
}
