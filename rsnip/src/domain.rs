use anyhow::Result;
use std::fs;
use std::path::Path;

pub struct CompletionType {
    pub name: String,
    pub source_file: std::path::PathBuf,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Snippet {
    pub name: String,
    pub snippet: Option<String>,
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
