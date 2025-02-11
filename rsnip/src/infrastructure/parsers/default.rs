use crate::domain::content::SnippetContent;
use crate::domain::parser::SnippetParser;
use crate::domain::snippet::Snippet;
use anyhow::{Context, Result};
use std::path::Path;
use thiserror::Error;
use tracing::{debug, instrument};

#[derive(Error, Debug)]
pub enum ParserError {
    #[error("Malformed snippet '{name}' at line {line}: {reason}")]
    MalformedSnippet {
        name: String,
        line: usize,
        reason: String,
    },
}

pub struct DefaultSnippetParser;

impl DefaultSnippetParser {
    pub fn new() -> Self {
        Self
    }

    fn validate_snippet(name: &str, content_lines: &[String], line_number: usize) -> Result<()> {
        if content_lines.is_empty() {
            return Err(ParserError::MalformedSnippet {
                name: name.to_string(),
                line: line_number,
                reason: "Empty content".to_string(),
            }
            .into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
struct SnippetBuilder {
    name: Option<String>,
    content_lines: Vec<String>,
    comments: Vec<String>,
    start_line: usize,
}

impl SnippetBuilder {
    fn new() -> Self {
        Self {
            name: None,
            content_lines: Vec::new(),
            comments: Vec::new(),
            start_line: 0,
        }
    }

    fn build(self) -> Result<Option<Snippet>> {
        match self.name {
            Some(name) => {
                // Validate the snippet structure
                DefaultSnippetParser::validate_snippet(&name, &self.content_lines, self.start_line)?;

                let snippet_text = self.content_lines.join("\n");
                Ok(Some(Snippet {
                    name,
                    content: SnippetContent::new(snippet_text),
                    comments: self.comments,
                }))
            }
            None => Ok(None),
        }
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
        let mut in_snippet = false;
        let mut last_line_empty = false;

        for (line_num, line) in content.lines().enumerate() {
            let line_num = line_num + 1; // Convert to 1-based line numbers
            let trimmed = line.trim();

            if trimmed.starts_with("--- ") {
                // If we're already in a snippet, this means we found a new one without proper closure
                if in_snippet {
                    return Err(ParserError::MalformedSnippet {
                        name: builder.name.unwrap_or_default(),
                        line: builder.start_line,
                        reason: "Missing closing delimiter (---)".to_string(),
                    }
                    .into());
                }

                // Start new snippet
                builder = SnippetBuilder::new();
                let name = trimmed.trim_start_matches("--- ").to_string();
                builder.name = Some(name);
                builder.start_line = line_num;
                in_snippet = true;
                last_line_empty = false;
            } else if trimmed == "---" {
                if !in_snippet {
                    return Err(ParserError::MalformedSnippet {
                        name: "unknown".to_string(),
                        line: line_num,
                        reason: "Found closing delimiter without opening snippet".to_string(),
                    }
                    .into());
                }

                // Finalize current snippet
                if let Some(snippet) = builder.build()? {
                    snippets.push(snippet);
                }
                builder = SnippetBuilder::new();
                in_snippet = false;
                last_line_empty = false;
            } else if in_snippet {
                // Handle content or comment
                if trimmed.starts_with(':') {
                    builder.comments.push(trimmed[1..].trim().to_string());
                } else {
                    // Only add empty lines if they're not at the start/end and not consecutive
                    if !trimmed.is_empty() || (!builder.content_lines.is_empty() && !last_line_empty) {
                        builder.content_lines.push(line.to_string());
                    }
                    last_line_empty = trimmed.is_empty();
                }
            }
        }

        // Handle unclosed snippet at EOF
        if in_snippet {
            return Err(ParserError::MalformedSnippet {
                name: builder.name.unwrap_or_default(),
                line: builder.start_line,
                reason: "Missing closing delimiter (---) at end of file".to_string(),
            }
            .into());
        }

        Ok(snippets)
    }
}