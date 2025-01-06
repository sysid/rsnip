use anyhow::{anyhow, Result};
use crate::application::{copy_snippet_to_clipboard, find_completion_exact, find_completion_interactive};
use crate::cli::args::{Cli, Commands};
use crate::domain::SnippetType;

pub fn execute_command(cli: &Cli) -> Result<()> {
    match &cli.command {
        Some(Commands::Complete {
                 ctype,
                 input,
                 interactive,
             }) => {
            let completion_type = SnippetType {
                name: ctype.to_string(),
                source_file: std::path::PathBuf::from("completion_source.txt"),
            };

            let input = input.as_deref().unwrap_or("");

            if *interactive {
                if let Some(item) = find_completion_interactive(&completion_type, input)? {
                    println!("{}", item.name);
                }
            } else if let Some(item) = find_completion_exact(&completion_type, input)? {
                println!("{}", item.name);
            }
            Ok(())
        }
        Some(Commands::Xxx { ctype, input }) => {
            // Placeholder for future command
            println!("Command xxx: {} {}", ctype, input);
            Ok(())
        }
        Some(Commands::Copy { ctype, input }) => {
            let completion_type = SnippetType {
                name: ctype.clone(),
                source_file: std::path::PathBuf::from("completion_source.txt"),
            };

            match copy_snippet_to_clipboard(&completion_type, input, true)? {
                Some(snippet) => {
                    println!("Snippet '{}' copied to clipboard:\n", snippet.name);
                    println!("{}", snippet.snippet.unwrap_or_default());
                    Ok(())
                }
                None => {
                    Err(anyhow!("No matching snippet found for '{}'", input))
                }
            }
        }
        None => Ok(())
    }
}