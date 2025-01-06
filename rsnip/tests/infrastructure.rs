use std::io::Write;
use tempfile::NamedTempFile;
use rsnip::domain::Snippet;
use rsnip::infrastructure::parse_snippets_file;

#[test]
fn given_valid_snippet_file_when_parse_snippets_file_then_returns_correct_snippets() {
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

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    writeln!(temp_file, "{}", content).expect("Failed to write test content");
    let path = temp_file.path();

    // Act
    let snippets = parse_snippets_file(path).expect("Should parse successfully");

    // Assert
    // We expect four snippets: apple, aple, banana, else
    assert_eq!(snippets.len(), 4);

    assert_eq!(
        snippets[0],
        Snippet {
            name: "apple".to_string(),
            snippet: Some("this is green\nand nothing else".to_string())
        }
    );
    assert_eq!(
        snippets[1],
        Snippet {
            name: "aple".to_string(),
            snippet: Some("this is green2".to_string())
        }
    );
    assert_eq!(
        snippets[2],
        Snippet {
            name: "banana".to_string(),
            snippet: Some("this is yellow".to_string())
        }
    );
    assert_eq!(
        snippets[3],
        Snippet {
            name: "else".to_string(),
            snippet: Some("this is other".to_string())
        }
    );
}

#[test]
fn given_snippet_file_missing_trailing_delimiter_when_parse_snippets_file_then_still_returns_snippet(
) {
    // Arrange
    let content = r#"
--- apple
line1
line2
"#;
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    writeln!(temp_file, "{}", content).expect("Failed to write test content");
    let path = temp_file.path();

    // Act
    let snippets = parse_snippets_file(path).expect("Should parse successfully");

    // Assert
    // One snippet, never ended but finalize at EOF
    assert_eq!(snippets.len(), 1);
    assert_eq!(
        snippets[0],
        Snippet {
            name: "apple".to_string(),
            snippet: Some("line1\nline2\n".to_string())
        }
    );
}

#[test]
fn given_empty_file_when_parse_snippets_file_then_returns_empty_vec() {
    // Arrange
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let path = temp_file.path();

    // Act
    let snippets = parse_snippets_file(path).expect("Should parse successfully");

    // Assert
    assert!(snippets.is_empty());
}

#[test]
fn given_snippet_file_extra_text_outside_snippets_when_parse_snippets_file_then_ignores_it() {
    // Arrange
    let content = r#"
random text
--- apple
line
---
random trailing text
"#;
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    writeln!(temp_file, "{}", content).expect("Failed to write test content");
    let path = temp_file.path();

    // Act
    let snippets = parse_snippets_file(path).expect("Should parse successfully");

    // Assert
    // The "random text" outside snippet blocks is just ignored
    assert_eq!(snippets.len(), 1);
    assert_eq!(
        snippets[0],
        Snippet {
            name: "apple".to_string(),
            snippet: Some("line".to_string())
        }
    );
}
