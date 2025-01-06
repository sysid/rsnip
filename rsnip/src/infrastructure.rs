use crate::domain::Snippet;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use arboard::Clipboard;

pub fn read_completions_from_file(path: &Path) -> Result<Vec<Snippet>> {
    let content = fs::read_to_string(path)?;
    let items = content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }
            let mut parts = line.splitn(2, '|');
            let text = parts.next()?.to_string();
            let desc = parts.next().map(|s| s.to_string());
            Some(Snippet {
                name: text,
                snippet: desc,
            })
        })
        .collect();
    Ok(items)
}

/// Copy text to system clipboard, removing any trailing newlines
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().context("Failed to initialize clipboard")?;
    // Ensure we remove any trailing newlines
    let clean_text = text.trim_end_matches('\n');
    clipboard.set_text(clean_text).context("Failed to set clipboard text")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_text_with_single_newline_when_copy_to_clipboard_then_newline_is_removed() -> Result<()> {
        let mut clipboard = Clipboard::new()?;
        let text = "Hello\n";
        copy_to_clipboard(text)?;
        assert_eq!(clipboard.get_text()?, "Hello");
        Ok(())
    }

    #[test]
    fn given_text_with_multiple_newlines_when_copy_to_clipboard_then_newlines_are_removed() -> Result<()> {
        let mut clipboard = Clipboard::new()?;
        let text = "Hello\nWorld\n\n";
        copy_to_clipboard(text)?;
        assert_eq!(clipboard.get_text()?, "Hello\nWorld");
        Ok(())
    }

    #[test]
    fn given_text_with_no_newline_when_copy_to_clipboard_then_text_remains_unchanged() -> Result<()> {
        let mut clipboard = Clipboard::new()?;
        let text = "Hello";
        copy_to_clipboard(text)?;
        assert_eq!(clipboard.get_text()?, "Hello");
        Ok(())
    }

    #[test]
    fn given_empty_text_when_copy_to_clipboard_then_clipboard_is_empty() -> Result<()> {
        let mut clipboard = Clipboard::new()?;
        copy_to_clipboard("")?;
        assert_eq!(clipboard.get_text()?, "");
        Ok(())
    }
}

pub fn parse_snippets_file(path: &Path) -> Result<Vec<Snippet>> {
    let content = fs::read_to_string(path)?;
    let mut snippets = Vec::new();

    let mut current_name: Option<String> = None;
    let mut current_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        // Check if the line starts a new snippet block: `--- something`
        if trimmed.starts_with("--- ") {
            // If we already have an open snippet, finalize it first
            if let Some(name) = current_name.take() {
                let snippet_text = current_lines.join("\n");
                snippets.push(Snippet {
                    name,
                    snippet: if snippet_text.is_empty() { None } else { Some(snippet_text) },
                });
            }

            // Extract snippet name
            let name = trimmed.trim_start_matches("--- ").to_string();
            current_name = Some(name);
            current_lines.clear();

        } else if trimmed == "---" {
            // This signals the end of the current snippet block
            if let Some(name) = current_name.take() {
                let snippet_text = current_lines.join("\n");
                snippets.push(Snippet {
                    name,
                    snippet: if snippet_text.is_empty() { None } else { Some(snippet_text) },
                });
            }
            current_lines.clear();

        } else {
            // If inside a snippet, accumulate lines
            if current_name.is_some() {
                current_lines.push(line.to_string());
            }
        }
    }

    // If file ends without a trailing `---`
    if let Some(name) = current_name.take() {
        let snippet_text = current_lines.join("\n");
        snippets.push(Snippet {
            name,
            snippet: if snippet_text.is_empty() { None } else { Some(snippet_text) },
        });
    }

    Ok(snippets)
}