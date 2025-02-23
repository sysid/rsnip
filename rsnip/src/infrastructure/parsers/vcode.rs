// infrastructure/parsers/vcode.rs
use crate::domain::content::SnippetContent;
use crate::domain::parser::SnippetParser;
use crate::domain::snippet::Snippet;
use crate::domain::errors::{SnippetError, SnippetResult};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, instrument};

#[derive(Debug, Deserialize)]
struct VCodeSnippet {
    prefix: String,
    body: SnippetBody,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SnippetBody {
    Single(String),
    Multiple(Vec<String>),
}

pub struct VCodeSnippetParser;

impl Default for VCodeSnippetParser {
    fn default() -> Self {
        Self::new()
    }
}

impl VCodeSnippetParser {
    pub fn new() -> Self {
        Self
    }

    #[allow(dead_code)]
    fn convert_placeholders(input: &str) -> String {
        // Convert VSCode style placeholders (${1:label} or $1) to Jinja2 style
        let re = regex::Regex::new(r"\$\{(\d+):([^}]+)\}|\$(\d+)").unwrap();
        re.replace_all(input, |caps: &regex::Captures| {
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
}

impl SnippetParser for VCodeSnippetParser {
    #[instrument(level = "debug", skip(self))]
    fn parse(&self, path: &Path) -> SnippetResult<Vec<Snippet>> {
        debug!("Parsing VSCode format snippets from: {:?}", path);

        let content = std::fs::read_to_string(path)
            .map_err(|e| SnippetError::FileError {
                file: path.to_path_buf(),
                source: e,
            })?;

        let snippets: HashMap<String, VCodeSnippet> = serde_json::from_str(&content)
            .map_err(|e| SnippetError::InvalidFormat {
                name: "".to_string(),
                file: path.to_path_buf(),
                line: 1, // JSON errors don't provide line numbers
                reason: format!("Failed to parse JSON: {}", e),
            })?;

        let result = snippets
            .into_iter()
            .map(|(_, snippet)| {
                let body = match snippet.body {
                    SnippetBody::Single(text) => text,
                    SnippetBody::Multiple(lines) => lines.join("\n"),
                };

                let comments = snippet.description.map_or_else(Vec::new, |desc| vec![desc]);

                Snippet {
                    name: snippet.prefix,
                    content: SnippetContent::new(body),
                    comments,
                }
            })
            .collect();

        Ok(result)
    }
}