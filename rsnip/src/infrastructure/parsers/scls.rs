use crate::domain::content::SnippetContent;
use crate::domain::parser::SnippetParser;
use crate::domain::snippet::Snippet;
use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
// infrastructure/parsers/scls.rs
use std::path::Path;
use tracing::{debug, instrument};

// Lazily initialized regex for placeholder conversion
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

    /// Convert VSCode-style placeholders to Jinja2 template syntax
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
    fn parse(&self, path: &Path) -> Result<Vec<Snippet>> {
        debug!("Parsing SCLS format snippets from: {:?}", path);

        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read SCLS snippets from '{}'", path.display()))?;

        let snippet_file: SclsSnippetFile = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML from '{}'", path.display()))?;

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
