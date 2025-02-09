// infrastructure/parsers/scls.rs
use std::path::Path;
use anyhow::{Context, Result};
use serde::Deserialize;
use tracing::{debug, instrument};
use regex::Regex;
use once_cell::sync::Lazy;
use crate::domain::parser::SnippetParser;
use crate::domain::tochange::{Snippet, SnippetContent};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn given_valid_scls_file_when_parsing_then_returns_correct_snippets() -> Result<()> {
        let content = r#"
[[snippets]]
prefix = "log"
scope = ["python"]
body = "print($1)"
description = "Simple print statement"

[[snippets]]
prefix = "func"
scope = ["python", "javascript"]
body = "def ${1:name}(${2:args}):\n    ${3:pass}"
"#;
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "{}", content)?;

        let parser = SclsSnippetParser::new();
        let snippets = parser.parse(temp_file.path())?;

        assert_eq!(snippets.len(), 2);

        // Check first snippet
        assert_eq!(snippets[0].name, "log");
        assert_eq!(snippets[0].content.get_content(), "print({{ param1 }})");
        assert_eq!(snippets[0].comments.len(), 2);
        assert_eq!(snippets[0].comments[0], "Simple print statement");
        assert_eq!(snippets[0].comments[1], "Scope: python");

        // Check second snippet
        assert_eq!(snippets[1].name, "func");
        assert_eq!(
            snippets[1].content.get_content(),
            "def {{ name }}({{ args }}):\n    {{ pass }}"
        );
        assert_eq!(snippets[1].comments.len(), 1);
        assert_eq!(snippets[1].comments[0], "Scope: python, javascript");

        Ok(())
    }

    #[test]
    fn given_empty_scls_file_when_parsing_then_returns_empty_vec() -> Result<()> {
        let content = "# Empty SCLS file";
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "{}", content)?;

        let parser = SclsSnippetParser::new();
        let snippets = parser.parse(temp_file.path())?;

        assert!(snippets.is_empty());
        Ok(())
    }

    #[test]
    fn given_multiline_array_body_when_parsing_then_handles_correctly() -> Result<()> {
        let content = r#"
[[snippets]]
prefix = "class"
body = [
    "class ${1:ClassName}:",
    "    def __init__(self):",
    "        ${2:pass}"
]"#;
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "{}", content)?;

        let parser = SclsSnippetParser::new();
        let snippets = parser.parse(temp_file.path())?;

        assert_eq!(snippets.len(), 1);
        assert_eq!(
            snippets[0].content.get_content(),
            "class {{ ClassName }}:\n    def __init__(self):\n        {{ pass }}"
        );
        Ok(())
    }

    #[test]
    fn test_placeholder_conversion() {
        let test_cases = vec![
            ("print($1)", "print({{ param1 }})"),
            ("log.info(${1:message})", "log.info({{ message }})"),
            (
                "def ${1:name}($2): ${3:pass}",
                "def {{ name }}({{ param2 }}): {{ param3 }}"
            ),
            ("$1: Any = $2", "{{ param1 }}: Any = {{ param2 }}"),
        ];

        for (input, expected) in test_cases {
            assert_eq!(SclsSnippetParser::convert_placeholders(input), expected);
        }
    }

    #[test]
    fn test_process_body() {
        // Test string with escaped newlines
        assert_eq!(
            SclsSnippetParser::process_body("line1\\nline2"),
            "line1\nline2"
        );

        // Test array format
        assert_eq!(
            SclsSnippetParser::process_body(r#"["line1", "line2"]"#),
            "line1\nline2"
        );

        // Test regular string
        assert_eq!(
            SclsSnippetParser::process_body("simple string"),
            "simple string"
        );
    }
}