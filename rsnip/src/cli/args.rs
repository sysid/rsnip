use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)] // Read from `Cargo.toml`
#[command(arg_required_else_help = true, disable_help_subcommand = true)]
pub struct Cli {
    /// Enable debug logging. Multiple flags (-d, -dd, -ddd) increase verbosity
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    /// Generate shell completion scripts
    #[arg(long = "generate", value_enum)]
    pub generator: Option<Shell>,

    /// Print default configuration to stdout
    #[arg(long = "generate-config")]
    pub generate_config: bool,

    /// Display version and configuration information
    #[arg(long = "info")]
    pub info: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List available snippet types
    Types {
        /// Output space-separated list format
        #[arg(long)]
        list: bool,
    },
    /// List all snippets
    List {
        /// Type of snippets to list
        #[arg(long)]
        ctype: Option<String>,
    },
    /// Edit snippet in system editor
    Edit {
        /// Type of completion
        #[arg(long)]
        ctype: Option<String>,
        /// The snippet to edit
        #[arg(long)]
        input: Option<String>,
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn given_generate_config_flag_when_parsing_then_sets_flag() {
        let args = Cli::parse_from(["rsnip", "--generate-config"]);
        assert!(args.generate_config);
    }
}
