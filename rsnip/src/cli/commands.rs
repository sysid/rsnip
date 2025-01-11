use crate::application::{
    copy_snippet_to_clipboard, find_completion_exact, find_completion_interactive,
};
use crate::cli::args::{Cli, Commands};
use anyhow::{anyhow, Result};
use crossterm::style::Stylize;
use std::fs;

use crate::config::{get_snippet_type, Settings};
use crate::infrastructure;
use crate::infrastructure::parse_snippets_file;
use crate::path_utils::expand_path;

pub fn execute_command(cli: &Cli, config: &Settings) -> Result<()> {
    match &cli.command {
        Some(Commands::List { ctype }) => {
            let ctype = ctype.as_deref().unwrap_or("default");
            let snippet_type = get_snippet_type(config, ctype)?;
            let snippets = parse_snippets_file(&snippet_type.source_file)?;

            println!("\nSnippets for type '{}':", ctype);
            for snippet in snippets {
                let preview = snippet
                    .content
                    .get_content()
                    .lines()
                    .next()
                    .unwrap_or("")
                    .trim();

                if preview.is_empty() {
                    println!("  {}", snippet.name);
                } else {
                    println!("  {}: {}", snippet.name, preview);
                }
            }
            Ok(())
        }
        Some(Commands::Edit { ctype }) => {
            let ctype = ctype.as_deref().unwrap_or("default");
            let snippet_type = get_snippet_type(config, ctype)?;
            let expanded_path = expand_path(&snippet_type.source_file)?;

            // Create parent directories if they don't exist
            if let Some(parent) = expanded_path.parent() {
                fs::create_dir_all(parent)?;
            }

            // Create file if it doesn't exist
            if !expanded_path.exists() {
                println!(
                    "{}",
                    format!("Creating new snippet file: {}", expanded_path.display()).green()
                );
                fs::write(
                    &expanded_path,
                    "# Snippet file for type: ".to_string() + ctype,
                )?;
            }

            infrastructure::edit_snips_file(&snippet_type, Some(1usize))?;
            Ok(())
        }
        Some(Commands::Types) => {
            println!("\nAvailable snippet types:");
            for (name, cfg) in &config.snippet_types {
                if let Some(desc) = &cfg.description {
                    println!("  {}: {}", name, desc);
                } else {
                    println!("  {}", name);
                }
            }
            Ok(())
        }
        Some(Commands::Complete {
            ctype,
            input,
            interactive,
        }) => {
            let ctype = ctype.as_deref().unwrap_or("default");
            let snippet_type = get_snippet_type(config, ctype)?;
            let input = input.as_deref().unwrap_or("");

            if *interactive {
                if let Some(item) = find_completion_interactive(&snippet_type, input)? {
                    println!("{}", item.name);
                }
            } else if let Some(item) = find_completion_exact(&snippet_type, input)? {
                println!("{}", item.name);
            }
            Ok(())
        }
        Some(Commands::Xxx { ctype, input }) => {
            let ctype = ctype.as_deref().unwrap_or("default");
            let snippet_type = get_snippet_type(config, ctype)?;
            println!("Command xxx: {} {}", snippet_type.name, input);
            Ok(())
        }
        Some(Commands::Copy { ctype, input }) => {
            let ctype = ctype.as_deref().unwrap_or("default");
            let snippet_type = get_snippet_type(config, ctype)?;

            match copy_snippet_to_clipboard(&snippet_type, input, true)? {
                Some((snippet, rendered_content)) => {
                    println!(
                        "{}:\n{}\n",
                        "Comments:".to_string().yellow(),
                        snippet.comments.join("\n"),
                    );
                    println!(
                        "{}",
                        format!("'{}' -> clipboard:", snippet.name).green()
                    );
                    println!("{}", rendered_content);
                    Ok(())
                }
                None => Err(anyhow!("No matching snippet found for '{}'", input)),
            }
        }
        None => Ok(()),
    }
}
