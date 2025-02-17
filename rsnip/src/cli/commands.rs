use crate::application::snippet_service::SnippetService;
use crate::cli::args::{Cli, Commands};
use crate::config::{get_snippet_type, Settings, SnippetTypeConfig};
use crate::infrastructure::edit_snippets::{edit_snips_file, find_snippet_line_number};
use crate::infrastructure::minijinja::{MiniJinjaEngine, SafeShellExecutor};
use crate::util::path_utils::expand_path;
use anyhow::{anyhow, Result};
use crossterm::style::Stylize;
use itertools::Itertools;
use std::fs;
use dialoguer::Select;
use dialoguer::theme::ColorfulTheme;
use tracing::debug;

pub fn execute_command(cli: &Cli, config: &Settings) -> Result<()> {
    let template_engine = Box::new(MiniJinjaEngine::new(Box::new(SafeShellExecutor::new())));
    let service = SnippetService::new(template_engine, config);

    match &cli.command {
        Some(Commands::List { ctype, prefix }) => {
            let ctype = ctype.as_deref().unwrap_or("default");
            let mut snippets = service.get_snippets(ctype)?;

            // Filter by prefix if provided
            if let Some(prefix) = prefix {
                snippets.retain(|s| s.name.starts_with(prefix));
            }

            // Sort snippets by name
            snippets.sort_by(|a, b| a.name.cmp(&b.name));

            // Find the longest name for padding
            let max_name_len = snippets.iter().map(|s| s.name.len()).max().unwrap_or(0);
            debug!("Max name length: {}", max_name_len);

            eprintln!("\nSnippets for type '{}':", ctype); // Print to stderr for piping
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
                println!("  {}    {}", padded_name.green(), display_content);
            }
            Ok(())
        }
        Some(Commands::Edit { ctype, input }) => {
            let ctype = ctype.as_deref().unwrap_or("default");

            // Check if this is a combined type
            if let Some(sources) = config.get_combined_sources(ctype) {
                debug!("Handling combined type with sources: {:?}", sources);

                if let Some(input_name) = input {
                    // Search for the snippet in all source files
                    for source_name in &sources {
                        if let Some(source_type) = config.get_snippet_type(source_name) {
                            let file_path = expand_path(&source_type.source_file)?;
                            if file_path.exists() {
                                if let Ok(content) = fs::read_to_string(&file_path) {
                                    if let Some(line_number) =
                                        find_snippet_line_number(&content, input_name)  // todo: make it work for other formats
                                    {
                                        // Found the snippet, edit this file
                                        return edit_snips_file(&source_type, Some(line_number));
                                    }
                                }
                            }
                        }
                    }

                    // Snippet not found in any source, ask user which file to edit
                    println!("Snippet '{}' not found. Choose a file to edit:", input_name);
                } else {
                    println!("Choose a file to edit:");
                }

                // Collect valid source files
                let valid_sources: Vec<_> = sources
                    .iter()
                    .filter_map(|name| config.get_snippet_type(name).map(|st| (name, st)))
                    .collect();

                if valid_sources.is_empty() {
                    return Err(anyhow!("No valid source files found for type '{}'", ctype));
                }

                // Create selection items with file paths
                let items: Vec<String> = valid_sources
                    .iter()
                    .map(|(name, st)| format!("{}: {}", name, st.source_file.display()))
                    .collect();

                // Show selection dialog
                let selection = Select::with_theme(&ColorfulTheme::default())
                    .with_prompt("Select file to edit")
                    .items(&items)
                    .default(0)
                    .interact()?;

                // Edit the selected file
                let (_, snippet_type) = &valid_sources[selection];
                edit_snips_file(snippet_type, Some(1))?;
                return Ok(());
            }

            // Handle concrete type (existing logic)
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

            // If input is provided, find the snippet to edit
            let line_number = if let Some(input) = input {
                // Get snippet to find its position
                let snippets = service.get_snippets(ctype)?;
                snippets
                    .iter()
                    .position(|s| s.name == *input)
                    .map(|pos| pos + 1)
            } else {
                Some(1usize)
            };

            edit_snips_file(&snippet_type, line_number)?;
            Ok(())
        }

        Some(Commands::Types { list }) => {
            if *list {
                // Just print the types space-separated
                println!(
                    "{}",
                    config
                        .snippet_types
                        .keys()
                        .cloned()
                        .sorted()
                        .collect::<Vec<_>>()
                        .join(" ")
                );
            } else {
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
            Ok(())
        }
        Some(Commands::Complete {
            ctype,
            input,
            interactive,
        }) => {
            let ctype = ctype.as_deref().unwrap_or("default");
            let input = input.as_deref().unwrap_or("");

            if *interactive {
                if let Some(item) = service.find_completion_interactive(ctype, input)? {
                    println!("{}", item.name);
                }
            } else if let Some(item) = service.find_completion_exact(ctype, input)? {
                println!("{}", item.name);
            }
            Ok(())
        }
        Some(Commands::Copy { ctype, input }) => {
            let ctype = ctype.as_deref().unwrap_or("default");

            match service.copy_snippet_to_clipboard(ctype, input, true)? {
                Some((snippet, rendered_content)) => {
                    // only print comments if they exist
                    if !snippet.comments.is_empty() {
                        println!(
                            "{}:\n{}\n",
                            "Comments".to_string().yellow(),
                            snippet.comments.join("\n"),
                        );
                    }
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
