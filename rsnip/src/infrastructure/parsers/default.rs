use crate::domain::parser::SnippetParser;
use crate::domain::tochange::{Snippet, SnippetContent};
use anyhow::{Context, Result};
use std::path::Path;
use tracing::{debug, instrument};

pub struct DefaultSnippetParser;

impl DefaultSnippetParser {
    pub fn new() -> Self {
        Self
    }
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

impl SnippetParser for DefaultSnippetParser {
    #[instrument(level = "debug", skip(self))]
    fn parse(&self, path: &Path) -> Result<Vec<Snippet>> {
        debug!("Parsing default format snippets from: {:?}", path);
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read snippets from '{}'", path.display()))?;

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

        Ok(snippets)
    }
}
