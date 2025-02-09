mod scls_parser;

use crate::domain::{Snippet, SnippetContent, SnippetType};
use anyhow::{anyhow, Context, Result};
use arboard::Clipboard;
use crossterm::style::Stylize;
use std::path::Path;
use std::process::Command;
use std::{env, fs};
use tracing::instrument;

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

#[derive(Clone, Debug)]
struct SnippetBuilder {
    name: Option<String>,
    content_lines: Vec<String>,
    comments: Vec<String>,
}

impl SnippetBuilder {
    fn new() -> Self {
        Self {
            name: None,
            content_lines: Vec::new(),
            comments: Vec::new(),
        }
    }

    fn build(self) -> Option<Snippet> {
        self.name.map(|name| {
            let snippet_text = self.content_lines.join("\n");
            Snippet {
                name,
                content: SnippetContent::new(snippet_text),
                comments: self.comments,
            }
        })
    }
}

#[instrument(level = "trace")]
fn parse_content(content: &str) -> Vec<Snippet> {
    let mut snippets = Vec::new();
    let mut builder = SnippetBuilder::new();

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("--- ") {
            // Finalize previous snippet if exists
            if let Some(snippet) = builder.build() {
                snippets.push(snippet);
            }

            // Start new snippet
            builder = SnippetBuilder::new();
            let name = trimmed.trim_start_matches("--- ").to_string();
            builder.name = Some(name);
        } else if trimmed == "---" {
            // Finalize current snippet
            if let Some(snippet) = builder.build() {
                snippets.push(snippet);
            }
            builder = SnippetBuilder::new();
        } else if builder.name.is_some() {
            // Handle content or comment
            if trimmed.starts_with(':') {
                builder.comments.push(trimmed[1..].trim().to_string());
            } else {
                builder.content_lines.push(line.to_string());
            }
        }
    }

    // Handle last snippet if exists
    if let Some(snippet) = builder.build() {
        snippets.push(snippet);
    }

    snippets
}

#[instrument(level = "debug")]
pub fn parse_snippets_file(path: &Path) -> Result<Vec<Snippet>> {
    if !path.exists() {
        eprintln!("{}", format!("Warning: Snippet file '{}' not found, using default.\nYou need to create yours!\n", path.display()).red());
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

/// Find line number where a snippet starts
#[instrument(level = "debug")]
pub fn find_snippet_line_number(content: &str, snippet_name: &str) -> Option<usize> {
    content
        .lines()
        .enumerate()
        .find(|(_, line)| line.trim() == format!("--- {}", snippet_name))
        .map(|(idx, _)| idx + 1)
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
