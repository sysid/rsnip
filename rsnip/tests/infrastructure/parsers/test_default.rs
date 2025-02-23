use anyhow::Result;
use rsnip::domain::content::SnippetContent;
use rsnip::domain::parser::SnippetFormat;
use rsnip::domain::snippet::Snippet;
use rsnip::infrastructure::parsers::SnippetParserFactory;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[test]
fn given_valid_snippet_file_when_parse_then_returns_correct_snippets() -> Result<()> {
    // Arrange
    let content = r#"
: This is a completion source file

--- apple
this is green
and nothing else
---

--- aple
this is green2
---

--- banana
this is yellow
---
--- else
this is other
---
"#;

    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = SnippetParserFactory::create(SnippetFormat::Default);

    // Act
    let snippets = parser.parse(temp_file.path())?;

    // Assert
    assert_eq!(snippets.len(), 4);
    assert_eq!(
        snippets[0],
        Snippet {
            name: "apple".to_string(),
            content: SnippetContent::Static("this is green\nand nothing else".to_string()),
            comments: vec![]
        }
    );
    assert_eq!(
        snippets[1],
        Snippet {
            name: "aple".to_string(),
            content: SnippetContent::Static("this is green2".to_string()),
            comments: vec![]
        }
    );
    assert_eq!(
        snippets[2],
        Snippet {
            name: "banana".to_string(),
            content: SnippetContent::Static("this is yellow".to_string()),
            comments: vec![]
        }
    );
    assert_eq!(
        snippets[3],
        Snippet {
            name: "else".to_string(),
            content: SnippetContent::Static("this is other".to_string()),
            comments: vec![]
        }
    );
    Ok(())
}

#[test]
fn given_missing_trailing_delimiter_when_parse_then_returns_error() -> Result<()> {
    // Arrange
    let content = r#"
--- apple
line1
line2"#;
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = SnippetParserFactory::create(SnippetFormat::Default);

    // Act
    let result = parser.parse(temp_file.path());

    // Assert
    let err = result.unwrap_err().to_string();
    println!("Actual error message: {}", err);  // Debug print
    assert!(
        err.contains("Missing closing delimiter"),
        "Error should explain missing delimiter"
    );
    assert!(
        err.contains("line 2"),
        "Error should indicate the line number"
    );
    Ok(())
}

#[test]
fn given_empty_file_when_parse_then_returns_empty_vec() -> Result<()> {
    // Arrange
    let temp_file = NamedTempFile::new()?;
    let parser = SnippetParserFactory::create(SnippetFormat::Default);

    // Act
    let snippets = parser.parse(temp_file.path())?;

    // Assert
    assert!(snippets.is_empty());
    Ok(())
}

#[test]
fn given_extra_text_outside_snippets_when_parse_then_ignores_it() -> Result<()> {
    // Arrange
    let content = r#"
random text
--- apple
line
---
random trailing text
"#;
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = SnippetParserFactory::create(SnippetFormat::Default);

    // Act
    let snippets = parser.parse(temp_file.path())?;

    // Assert
    assert_eq!(snippets.len(), 1);
    assert_eq!(
        snippets[0],
        Snippet {
            name: "apple".to_string(),
            content: SnippetContent::Static("line".to_string()),
            comments: vec![]
        }
    );
    Ok(())
}

#[test]
fn given_snippet_with_comments_when_parse_then_preserves_comments() -> Result<()> {
    // Arrange
    let content = r#"
: File level comment (ignored)
--- apple
: This is a comment about apples
: Another comment
this is green
and nothing else
---"#;
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = SnippetParserFactory::create(SnippetFormat::Default);

    // Act
    let snippets = parser.parse(temp_file.path())?;

    // Assert
    assert_eq!(snippets.len(), 1);
    let snippet = &snippets[0];
    assert_eq!(snippet.name, "apple");
    assert_eq!(
        snippet.comments,
        vec!["This is a comment about apples", "Another comment"]
    );
    assert_eq!(
        snippet.content,
        SnippetContent::Static("this is green\nand nothing else".to_string())
    );
    Ok(())
}

#[test]
fn given_nonexistent_file_when_parse_then_returns_error() -> Result<()> {
    // Arrange
    let path = PathBuf::from("/nonexistent/file");
    let parser = SnippetParserFactory::create(SnippetFormat::Default);

    // Act
    let result = parser.parse(&path);

    // Assert
    assert!(result.is_err());
    Ok(())
}

#[test]
fn given_malformed_snippet_without_closing_delimiter_when_parse_then_returns_error() -> Result<()> {
    // Arrange
    let content = r#"
--- valid
content
---
--- malformed
: missing end delimiter
"#;
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = SnippetParserFactory::create(SnippetFormat::Default);

    // Act
    let result = parser.parse(temp_file.path());

    // Assert
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    println!("Actual error message: {}", err); // Debug print
    assert!(
        err.contains("Missing closing delimiter"),
        "Error should explain the issue"
    );
    Ok(())
}

#[test]
fn given_malformed_snippet_with_extra_start_when_parse_then_returns_error() -> Result<()> {
    // Arrange
    let content = r#"
--- valid
content
---
--- malformed
--- another-start
content
---
"#;
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = SnippetParserFactory::create(SnippetFormat::Default);

    // Act
    let result = parser.parse(temp_file.path());

    // Assert
    let err = result.unwrap_err().to_string();
    println!("Actual error message: {}", err); // Debug print
    assert!(
        err.contains("Missing closing delimiter"),
        "Error should explain the issue"
    );
    Ok(())
}

#[test]
fn given_snippet_with_closing_without_opening_when_parse_then_returns_error() -> Result<()> {
    // Arrange
    let content = r#"
--- valid
content
---
---
content
---
"#;
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = SnippetParserFactory::create(SnippetFormat::Default);

    // Act
    let result = parser.parse(temp_file.path());

    // Assert
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Found closing delimiter without opening"),
        "Error should explain the issue"
    );
    Ok(())
}

#[test]
fn given_empty_snippet_when_parse_then_returns_error() -> Result<()> {
    // Arrange
    let content = r#"
--- empty
---
"#;
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = SnippetParserFactory::create(SnippetFormat::Default);

    // Act
    let result = parser.parse(temp_file.path());

    // Assert
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Empty content"),
        "Error should explain the issue"
    );
    Ok(())
}
