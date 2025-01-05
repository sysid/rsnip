use clap::{Parser, Subcommand};
use rsnip::application::{perform_completion, print_completions};
use rsnip::domain::CompletionType;

/// Command line options
#[derive(Debug, Parser)]
#[command(name = "autocomplete-poc", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Perform completion using a specified type and user input
    Complete {
        /// Type of completion e.g. "mytype"
        #[arg(short, long)]
        ctype: String,
        /// The partial input to match on
        #[arg(short, long)]
        input: String,
        /// If --scriptable-output is set, we print all matches instead of copying
        #[arg(long)]
        scriptable_output: bool,
    },
    Xxx {
        /// Type of completion e.g. "mytype"
        #[arg(short, long)]
        ctype: String,
        /// The partial input to match on
        #[arg(short, long)]
        input: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Complete {
            ctype,
            input,
            scriptable_output,
        } => {
            // For POC, let's define a single completion type directly:
            let completion_type = CompletionType {
                name: ctype,
                source_file: std::path::PathBuf::from("completion_source.txt"),
                keyboard_shortcut: "ctrl+x".into(),
                action: rsnip::domain::CompletionAction::CopyToClipboard,
            };

            if scriptable_output {
                // Print all matches in a script-friendly way
                print_completions(&completion_type, &input)?;
            } else {
                // Normal path: copy to clipboard, etc.
                match perform_completion(&completion_type, &input)? {
                    Some(item) => {
                        println!("Matched: {}", item.text);
                    }
                    None => println!("No match found."),
                }
            }
        },
        Commands::Xxx { ctype, input } => {
            // Placeholder for future command
            println!("Command xxx: {} {}", ctype, input);
        }
    }
    Ok(())
}
