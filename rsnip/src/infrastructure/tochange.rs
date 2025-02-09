use std::env;
use std::process::Command;
use anyhow::{anyhow, Context, Result};
use arboard::Clipboard;
use crossterm::style::Stylize;
use tracing::instrument;
use crate::domain::parser::SnippetType;
use crate::domain::tochange::{Snippet, SnippetContent};

// Embed default completions file at compile time
const DEFAULT_COMPLETIONS: &str = include_str!("../default_completions.txt");

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
