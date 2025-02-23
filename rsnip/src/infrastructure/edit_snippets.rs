use crate::domain::parser::SnippetType;
use anyhow::{anyhow, Context, Result};
use std::env;
use std::process::Command;
use tracing::instrument;

// Embed default completions file at compile time
#[allow(dead_code)]
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
                cmd.arg("--goto");
                cmd.arg(format!("{}:{}", snippet_type.source_file.display(), line));
            }
            "hx" => {
                cmd.arg(format!("+{}", line));
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

    #[test]
    fn given_valid_content_when_finding_snippet_then_returns_correct_line() -> Result<()> {
        let content = "some content\n--- test\ncontent\n---";
        assert_eq!(find_snippet_line_number(content, "test"), Some(2));
        Ok(())
    }

    #[test]
    fn given_nonexistent_snippet_when_finding_line_then_returns_none() -> Result<()> {
        let content = "some content\n--- test\ncontent\n---";
        assert_eq!(find_snippet_line_number(content, "nonexistent"), None);
        Ok(())
    }
}
