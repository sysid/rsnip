use clap::{Parser, Subcommand};
use rsnip::application::{copy_snippet_to_clipboard, find_completion_exact, find_completion_interactive};
use rsnip::domain::SnippetType;

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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Complete {
            ctype,
            input,
            interactive,
        } => {
            let completion_type = SnippetType {
                name: ctype,
                source_file: std::path::PathBuf::from("completion_source.txt"),
            };

            let input = input.as_deref().unwrap_or("");

            if interactive {
                if let Some(item) = find_completion_interactive(&completion_type, input)? {
                    println!("{}", item.name);
                }
            } else if let Some(item) = find_completion_exact(&completion_type, input)? {
                println!("{}", item.name);
            }
        }
        Commands::Xxx { ctype, input } => {
            // Placeholder for future command
            println!("Command xxx: {} {}", ctype, input);
        }
        Commands::Copy { ctype, input } => {
            let completion_type = SnippetType {
                name: ctype.clone(),
                source_file: std::path::PathBuf::from("completion_source.txt"),
            };

            match copy_snippet_to_clipboard(&completion_type, &input, true)? {
                Some(snippet) => {
                    println!("Snippet '{}' copied to clipboard:\n", snippet.name);
                    println!("{}", snippet.snippet.unwrap_or_default());
                }
                None => {
                    eprintln!("No matching snippet found for '{}'", input);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // https://docs.rs/clap/latest/clap/_derive/_tutorial/index.html#testing
    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
