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