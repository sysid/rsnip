// infrastructure/parsers/scls.rs
use crate::domain::content::SnippetContent;
use crate::domain::parser::SnippetParser;
use crate::domain::snippet::Snippet;
use crate::domain::errors::{SnippetError, SnippetResult};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::path::Path;
use tracing::{debug, instrument};

static PLACEHOLDER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\$\{(\d+):([^}]+)\}|\$(\d+)").expect("Failed to compile placeholder regex")
});

#[derive(Debug, Deserialize)]
struct SclsSnippetFile {
    #[serde(default)]
    snippets: Vec<SclsSnippet>,
}

#[derive(Debug, Deserialize)]
struct SclsSnippet {
    prefix: String,
    #[serde(default)]
    scope: Vec<String>,
    body: String,
    #[serde(default)]
    description: Option<String>,
}

pub struct SclsSnippetParser;

impl Default for SclsSnippetParser {
    fn default() -> Self {
        Self::new()
    }
}

impl SclsSnippetParser {
    pub fn new() -> Self {
        Self
    }

    fn convert_placeholders(input: &str) -> String {
        PLACEHOLDER_REGEX.replace_all(input, |caps: &regex::Captures| {
            if let Some(text) = caps.get(2) {
                format!("{{{{ {} }}}}", text.as_str())
            } else if let Some(num) = caps.get(3) {
                format!("{{{{ param{} }}}}", num.as_str())
            } else if let Some(num) = caps.get(1) {
                format!("{{{{ param{} }}}}", num.as_str())
            } else {
                "".to_string()
            }
        }).to_string()
    }

    /// Process multiline body text
    fn process_body(body: &str) -> String {
        // Handle both string and array body formats
        if body.starts_with('[') {
            // Try to parse as JSON array of strings
            if let Ok(lines) = serde_json::from_str::<Vec<String>>(body) {
                return lines.join("\n");
            }
        }
        // If not an array or parsing failed, treat as single string
        body.replace("\\n", "\n")
    }
}

impl SnippetParser for SclsSnippetParser {
    #[instrument(level = "debug", skip(self))]
    fn parse(&self, path: &Path) -> SnippetResult<Vec<Snippet>> {
        debug!("Parsing SCLS format snippets from: {:?}", path);

        let content = std::fs::read_to_string(path)
            .map_err(|e| SnippetError::FileError {
                file: path.to_path_buf(),
                source: e,
            })?;

        let snippet_file: SclsSnippetFile = toml::from_str(&content)
            .map_err(|e| SnippetError::InvalidFormat {
                name: "".to_string(),
                file: path.to_path_buf(),
                line: 1, // TOML errors don't provide line numbers
                reason: format!("Failed to parse TOML: {}", e),
            })?;

        let snippets = snippet_file.snippets.into_iter().map(|scls_snippet| {
            // Convert body text and handle placeholders
            let processed_body = Self::process_body(&scls_snippet.body);
            let body_with_placeholders = Self::convert_placeholders(&processed_body);

            // Collect comments (description and scope)
            let mut comments = Vec::new();
            if let Some(desc) = scls_snippet.description {
                comments.push(desc);
            }
            if !scls_snippet.scope.is_empty() {
                comments.push(format!("Scope: {}", scls_snippet.scope.join(", ")));
            }

            Snippet {
                name: scls_snippet.prefix,
                content: SnippetContent::new(body_with_placeholders),
                comments,
            }
        }).collect();

        Ok(snippets)
    }
}
