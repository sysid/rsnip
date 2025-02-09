use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;
use tracing::{debug, instrument};

use crate::domain::{Snippet, SnippetContent};

#[derive(Debug, Deserialize)]
struct SimpleSnippetFile {
    #[serde(default)]
    snippets: Vec<SimpleSnippet>,
}

#[derive(Debug, Deserialize)]
struct SimpleSnippet {
    prefix: String,
    #[serde(default)]
    scope: Vec<String>,
    body: String,
    #[serde(default)]
    description: Option<String>,
}

/// Parse simple-completion-language-server TOML snippet file
#[instrument(level = "debug")]
pub fn parse_simple_snippets_file(path: &Path) -> Result<Vec<Snippet>> {
    debug!("Parsing simple completion snippets from: {:?}", path);

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read snippets from '{}'", path.display()))?;

    let snippet_file: SimpleSnippetFile = toml::from_str(&content)
        .with_context(|| format!("Failed to parse TOML from '{}'", path.display()))?;

    // Convert SimpleSnippet format to rsnip Snippet format
    let mut result = Vec::new();
    for snippet in snippet_file.snippets {
        // Convert placeholder syntax from $1 to {{ param1 }}
        let body = convert_placeholders(&snippet.body);

        let mut comments = Vec::new();
        if let Some(desc) = snippet.description {
            comments.push(desc);
        }
        if !snippet.scope.is_empty() {
            comments.push(format!("Scope: {}", snippet.scope.join(", ")));
        }

        result.push(Snippet {
            name: snippet.prefix,
            content: SnippetContent::new(body),
            comments,
        });
    }

    Ok(result)
}

/// Convert VSCode-style placeholders to Jinja2 template syntax
fn convert_placeholders(input: &str) -> String {
    let mut result = input.to_string();

    // Match both ${number:text} and $number patterns
    let re = regex::Regex::new(r"\$\{(\d+):([^}]+)\}|\$(\d+)").unwrap();

    // Replace all matches with Jinja2 variables
    result = re.replace_all(&result, |caps: &regex::Captures| {
        if let Some(text) = caps.get(2) {
            format!("{{{{ {} }}}}", text.as_str())
        } else if let Some(num) = caps.get(3) {
            format!("{{{{ param{} }}}}", num.as_str())
        } else if let Some(num) = caps.get(1) {
            format!("{{{{ param{} }}}}", num.as_str())
        } else {
            "".to_string()
        }
    }).to_string();

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_simple_snippets_file() -> Result<()> {
        let content = r#"
[[snippets]]
prefix = "ld"
scope = ["python"]
body = 'log.debug("$1")'
description = "log at debug level"

[[snippets]]
prefix = "li"
scope = ["python", "rust"]
body = 'log.info("${1:message}")'
description = "log at info level"
"#;
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "{}", content)?;

        let snippets = parse_simple_snippets_file(temp_file.path())?;

        assert_eq!(snippets.len(), 2);

        // Check first snippet
        assert_eq!(snippets[0].name, "ld");
        assert_eq!(
            snippets[0].content.get_content(),
            "log.debug(\"{{ param1 }}\")"
        );
        assert_eq!(snippets[0].comments[0], "log at debug level");
        assert_eq!(snippets[0].comments[1], "Scope: python");

        // Check second snippet
        assert_eq!(snippets[1].name, "li");
        assert_eq!(
            snippets[1].content.get_content(),
            "log.info(\"{{ message }}\")"
        );
        assert_eq!(snippets[1].comments[0], "log at info level");
        assert_eq!(snippets[1].comments[1], "Scope: python, rust");

        Ok(())
    }

    #[test]
    fn test_convert_placeholders() {
        let cases = vec![
            ("log.debug(\"$1\")", "log.debug(\"{{ param1 }}\")"),
            ("print(\"${1:message}\")", "print(\"{{ message }}\")"),
            ("$1: $2", "{{ param1 }}: {{ param2 }}"),
            ("${1:arg}: ${2:type}", "{{ arg }}: {{ type }}"),
        ];

        for (input, expected) in cases {
            assert_eq!(convert_placeholders(input), expected);
        }
    }

    #[test]
    fn test_parse_empty_file() -> Result<()> {
        let test_cases = vec![
            "# Empty file\n",
            "",
            "# Just comments\n# Another comment",
            "[[wrong]]\nsome = \"content\"",  // Wrong structure but should not fail
        ];

        for content in test_cases {
            let mut temp_file = NamedTempFile::new()?;
            writeln!(temp_file, "{}", content)?;

            let snippets = parse_simple_snippets_file(temp_file.path())?;
            assert!(snippets.is_empty(), "Expected empty snippets for content: {}", content);
        }
        Ok(())
    }
}