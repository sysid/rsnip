use anyhow::Result;
use rsnip::domain::parser::SnippetFormat;
use rsnip::infrastructure::parsers::SnippetParserFactory;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[test]
fn given_valid_scls_file_when_parse_then_returns_snippets() -> Result<()> {
    // Arrange
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
    let parser = SnippetParserFactory::create(SnippetFormat::Scls);

    // Act
    let snippets = parser.parse(temp_file.path())?;

    // Assert
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
fn given_empty_scls_file_when_parse_then_returns_empty_vec() -> Result<()> {
    // Arrange
    let content = "# Empty SCLS file";
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = SnippetParserFactory::create(SnippetFormat::Scls);

    // Act
    let snippets = parser.parse(temp_file.path())?;

    // Assert
    assert!(snippets.is_empty());
    Ok(())
}

#[test]
fn given_multiline_array_body_when_parse_then_handles_correctly() -> Result<()> {
    // Arrange
    let content = r#"
[[snippets]]
prefix = "class"
body = """
class ${1:ClassName}:
    def __init__(self):
        ${2:pass}
"""
"#;
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = SnippetParserFactory::create(SnippetFormat::Scls);

    // Act
    let snippets = parser.parse(temp_file.path())?;

    // Assert
    assert_eq!(snippets.len(), 1);
    assert_eq!(
        snippets[0].content.get_content(),
        "class {{ ClassName }}:\n    def __init__(self):\n        {{ pass }}\n"
    );
    Ok(())
}

#[test]
fn given_malformed_toml_when_parse_then_returns_error() -> Result<()> {
    // Arrange
    let content = r#"
[[snippets]
malformed = toml
"#;
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = SnippetParserFactory::create(SnippetFormat::Scls);

    // Act
    let result = parser.parse(temp_file.path());

    // Assert
    assert!(result.is_err());
    Ok(())
}

#[test]
fn given_snippet_with_named_placeholders_when_parse_then_converts_correctly() -> Result<()> {
    // Arrange
    let content = r#"
[[snippets]]
prefix = "example"
body = "${1:first} ${2:second} $3"
"#;
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = SnippetParserFactory::create(SnippetFormat::Scls);

    // Act
    let snippets = parser.parse(temp_file.path())?;

    // Assert
    assert_eq!(snippets.len(), 1);
    assert_eq!(
        snippets[0].content.get_content(),
        "{{ first }} {{ second }} {{ param3 }}"
    );
    Ok(())
}

#[test]
fn given_nonexistent_file_when_parse_then_returns_error() -> Result<()> {
    // Arrange
    let path = PathBuf::from("/nonexistent/file");
    let parser = SnippetParserFactory::create(SnippetFormat::Scls);

    // Act
    let result = parser.parse(&path);

    // Assert
    assert!(result.is_err());
    Ok(())
}