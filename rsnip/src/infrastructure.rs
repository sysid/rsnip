use crate::domain::{Snippet, SnippetType};
use anyhow::{anyhow, Context, Result};
use arboard::Clipboard;
use crossterm::style::Stylize;
use std::{env, fs};
use std::path::Path;
use tracing::instrument;
use std::process::Command;

// Embed default completions file at compile time
const DEFAULT_COMPLETIONS: &str = include_str!("default_completions.txt");

#[instrument(level = "trace")]
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().context("Failed to initialize clipboard")?;
    let clean_text = text.trim_end_matches('\n');
    clipboard
        .set_text(clean_text)
        .context("Failed to set clipboard text")?;
    Ok(())
}

#[instrument(level = "trace")]
fn parse_content(content: &str) -> Vec<Snippet> {
    let mut snippets = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("--- ") {
            if let Some(name) = current_name.take() {
                let snippet_text = current_lines.join("\n");
                snippets.push(Snippet {
                    name,
                    snippet: if snippet_text.is_empty() {
                        None
                    } else {
                        Some(snippet_text)
                    },
                });
            }

            let name = trimmed.trim_start_matches("--- ").to_string();
            current_name = Some(name);
            current_lines.clear();
        } else if trimmed == "---" {
            if let Some(name) = current_name.take() {
                let snippet_text = current_lines.join("\n");
                snippets.push(Snippet {
                    name,
                    snippet: if snippet_text.is_empty() {
                        None
                    } else {
                        Some(snippet_text)
                    },
                });
            }
            current_lines.clear();
        } else if current_name.is_some() {
            current_lines.push(line.to_string());
        }
    }

    if let Some(name) = current_name.take() {
        let snippet_text = current_lines.join("\n");
        snippets.push(Snippet {
            name,
            snippet: if snippet_text.is_empty() {
                None
            } else {
                Some(snippet_text)
            },
        });
    }

    snippets
}

#[instrument(level = "debug")]
pub fn parse_snippets_file(path: &Path) -> Result<Vec<Snippet>> {
    if !path.exists() {
        eprintln!("{}", format!("Warning: Snippet file '{}' not found, using default completions.\nYou want to change this!\n", path.display()).red());
        return Ok(parse_content(DEFAULT_COMPLETIONS));
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read snippets from '{}'", path.display()))?;

    Ok(parse_content(&content))
}

#[instrument(level = "debug")]
pub fn edit_snips_file(snippet_type: &SnippetType, line_number: Option<usize>) -> Result<()> {
    // Get editor from environment or fall back to sensible defaults
    let editor = env::var("EDITOR")
        .or_else(|_| env::var("VISUAL"))
        .unwrap_or_else(|_| "vim".to_string());

    // Build command with optional line number
    let mut cmd = Command::new(&editor);
    cmd.arg(&snippet_type.source_file);

    // Add line number argument based on editor
    if let Some(line) = line_number {
        match editor.as_str() {
            "vim" | "nvim" => {
                cmd.arg(format!("+{}", line));
            }
            "emacs" => {
                cmd.arg(format!("+{}", line));
            }
            "nano" => {
                cmd.arg(format!("+{}", line));
            }
            "code" | "codium" => {
                cmd.arg(format!("--goto"));
                cmd.arg(format!("{}:{}", snippet_type.source_file.display(), line));
            }
            _ => {} // Other editors might not support line numbers
        }
    }

    // Open editor
    let status = cmd
        .status()
        .with_context(|| format!("Failed to run editor: {}", editor))?;

    if !status.success() {
        return Err(anyhow!("Editor {} exited with error", editor));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn given_nonexistent_path_when_parse_snippets_file_then_uses_default() -> Result<()> {
        let nonexistent_path = PathBuf::from("does_not_exist.txt");
        let result = parse_snippets_file(&nonexistent_path)?;
        assert!(!result.is_empty()); // Default completions should contain something
        Ok(())
    }

    #[test]
    fn given_empty_file_when_parse_snippets_file_then_returns_empty_vec() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let snippets = parse_snippets_file(temp_file.path())?;
        assert!(snippets.is_empty());
        Ok(())
    }

    #[test]
    fn given_valid_snippet_file_when_parse_snippets_file_then_returns_snippets() -> Result<()> {
        let content = r#"
--- test1
content1
---
--- test2
content2
---"#;
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "{}", content)?;

        let snippets = parse_snippets_file(temp_file.path())?;
        assert_eq!(snippets.len(), 2);
        assert_eq!(snippets[0].name, "test1");
        assert_eq!(snippets[1].name, "test2");
        Ok(())
    }

    // Clipboard tests...
}
