use crate::domain::CompletionItem;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn read_completions_from_file(path: &Path) -> Result<Vec<CompletionItem>> {
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
            Some(CompletionItem {
                text,
                description: desc,
            })
        })
        .collect();
    Ok(items)
}

/// Placeholder function for copying a string to clipboard.
/// On macOS or Linux, you'd integrate with an appropriate library.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    // e.g. with the 'clipboard' crate or system call
    println!("(Simulating clipboard copy) {}", text);
    Ok(())
}
