use crate::domain::content::SnippetContent;
use crate::domain::errors::{SnippetError, SnippetResult};
use crate::domain::parser::SnippetParser;
use crate::domain::snippet::Snippet;
use std::path::Path;
use tracing::{debug, instrument};

pub struct DefaultSnippetParser;

impl Default for DefaultSnippetParser {
    fn default() -> Self {
        Self::new()
    }
}

impl DefaultSnippetParser {
    pub fn new() -> Self {
        Self
    }

    fn validate_snippet(
        name: &str,
        content_lines: &[String],
        line_number: usize,
        file: &Path,
    ) -> SnippetResult<()> {
        if content_lines.is_empty() {
            return Err(SnippetError::InvalidFormat {
                name: name.to_string(),
                file: file.to_path_buf(),
                line: line_number,
                reason: "Empty content".to_string(),
            });
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

    fn build(self, file: &Path) -> SnippetResult<Option<Snippet>> {
        match self.name {
            Some(name) => {
                DefaultSnippetParser::validate_snippet(
                    &name,
                    &self.content_lines,
                    self.start_line,
                    file,
                )?;
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
    fn parse(&self, path: &Path) -> SnippetResult<Vec<Snippet>> {
        debug!("Parsing default format snippets from: {:?}", path);
        let content = std::fs::read_to_string(path).map_err(|e| SnippetError::FileError {
            file: path.to_path_buf(),
            source: e,
        })?;

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
                    let name = builder.name.as_deref().unwrap_or("unknown");
                    return Err(SnippetError::InvalidFormat {
                        name: name.to_string(),
                        file: path.to_path_buf(),
                        line: builder.start_line,
                        reason: "Missing closing delimiter (---)".to_string(),
                    });
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
                    return Err(SnippetError::InvalidFormat {
                        name: "".to_string(),
                        file: path.to_path_buf(),
                        line: line_num,
                        reason: "Found closing delimiter without opening snippet".to_string(),
                    });
                }

                if let Some(snippet) = builder.build(path)? {
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
                    if !trimmed.is_empty()
                        || (!builder.content_lines.is_empty() && !last_line_empty)
                    {
                        builder.content_lines.push(line.to_string());
                    }
                    last_line_empty = trimmed.is_empty();
                }
            }
        }

        // Handle unclosed snippet at EOF
        if in_snippet {
            let name = builder.name.as_deref().unwrap_or("unknown");
            return Err(SnippetError::InvalidFormat {
                name: name.to_string(),
                file: path.to_path_buf(),
                line: builder.start_line,
                reason: "Missing closing delimiter (---) at end of file".to_string(),
            });
        }

        Ok(snippets)
    }
}
