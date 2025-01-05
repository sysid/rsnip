use clap::{Parser, Subcommand};
use rsnip::application::{find_completion, find_completion_interactive};
use rsnip::domain::CompletionType;

#[derive(Debug, Parser)]
#[command(name = "rsnip", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
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
        #[arg(short, long)]
        input: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Complete { ctype, input, interactive } => {
            let completion_type = CompletionType {
                name: ctype,
                source_file: std::path::PathBuf::from("completion_source.txt"),
                keyboard_shortcut: "ctrl+x".into(),
                action: rsnip::domain::CompletionAction::CopyToClipboard, // Will be removed later
            };

            let input = input.as_deref().unwrap_or("");

            if interactive {
                if let Some(item) = find_completion_interactive(&completion_type, input)? {
                    println!("{}", item.text);
                }
            } else {
                if let Some(item) = find_completion(&completion_type, input)? {
                    println!("{}", item.text);
                }
            }
        }
        Commands::Xxx { ctype, input } => {
            // Placeholder for future command
            println!("Command xxx: {} {}", ctype, input);
        }
    }

    Ok(())
}