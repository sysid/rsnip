use crate::application::{
    copy_snippet_to_clipboard, find_completion_exact, find_completion_interactive,
};
use crate::cli::args::{Cli, Commands};
use anyhow::{anyhow, Result};
use crossterm::style::Stylize;
use std::fs;
use itertools::Itertools;
use tracing::debug;
use crate::config::{get_snippet_type, Settings};
use crate::infrastructure;
use crate::infrastructure::{find_snippet_line_number, parse_snippets_file};
use crate::path_utils::expand_path;

pub fn execute_command(cli: &Cli, config: &Settings) -> Result<()> {
    match &cli.command {
        Some(Commands::List { ctype, prefix }) => {
            let ctype = ctype.as_deref().unwrap_or("default");
            let snippet_type = get_snippet_type(config, ctype)?;
            let mut snippets = parse_snippets_file(&snippet_type.source_file)?;

            // Filter by prefix if provided
            if let Some(prefix) = prefix {
                snippets.retain(|s| s.name.starts_with(prefix));
            }

            // Sort snippets by name
            snippets.sort_by(|a, b| a.name.cmp(&b.name));

            // Find the longest name for padding
            let max_name_len = snippets
                .iter()
                .map(|s| s.name.len())
                .max()
                .unwrap_or(0);
            debug!("Max name length: {}", max_name_len);

            eprintln!("\nSnippets for type '{}':", ctype);  // Print to stderr for piping
            for snippet in snippets {
                let content = snippet
                    .content
                    .get_content()
                    .lines()
                    .collect::<Vec<_>>()
                    .join(" ");

                // Truncate long content for display
                let display_content = if content.len() > 100 {
                    format!("{}...", &content[..97])
                } else {
                    content
                };

                // Using format! first to ensure the name padding works correctly
                let padded_name = format!("{:width$}", snippet.name, width = max_name_len);
                println!(
                    "  {}    {}",
                    padded_name.green(),
                    display_content
                );
            }
            Ok(())
        }
    Some(Commands::Edit { ctype, input }) => {
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

        // If input is provided, find the line number for that snippet
        let line_number = if let Some(input) = input {
            // let snippets = parse_snippets_file(&snippet_type.source_file)?;
            let content = fs::read_to_string(&expanded_path)?;

            // Find the snippet and its line number
            find_snippet_line_number(&content, &input)
        } else {
            Some(1usize)
        };

        infrastructure::edit_snips_file(&snippet_type, line_number)?;
        Ok(())
    },
        Some(Commands::Types { list }) => {
            if *list {
                // Just print the types space-separated
                println!("{}", config.snippet_types.keys().cloned().sorted().collect::<Vec<_>>().join(" "));
            } else {
                println!("\nAvailable snippet types:");
                for (name, cfg) in &config.snippet_types {
                    if let Some(desc) = &cfg.description {
                        println!("  {}: {}", name, desc);
                    } else {
                        println!("  {}", name);
                    }
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
                    println!("{}", format!("'{}' -> clipboard:", snippet.name).green());
                    println!("{}", rendered_content);
                    Ok(())
                }
                None => Err(anyhow!("No matching snippet found for '{}'", input)),
            }
        }
        None => Ok(()),
    }
}
