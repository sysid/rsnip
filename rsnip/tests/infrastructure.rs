use rsnip::cli::args::{Cli, Commands};
use rsnip::cli::commands::execute_command;
use rsnip::config::{Settings, SnippetTypeConfig};
use rsnip::domain::{Snippet, SnippetContent};
use rsnip::infrastructure::parse_snippets_file;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use tempfile::{tempdir, NamedTempFile};

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
            content: SnippetContent::Static("line1\nline2\n".to_string()),
            comments: vec![]
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
            content: SnippetContent::Static("line".to_string()),
            comments: vec![]
        }
    );
}

#[test]
fn given_nonexistent_file_when_edit_then_creates_file() -> anyhow::Result<()> {
    // Arrange
    let temp_dir = tempdir()?;
    let file_path = temp_dir.path().join("test_snippets.txt");

    let config = Settings {
        snippet_types: {
            let mut map = HashMap::new();
            map.insert(
                "test".to_string(),
                SnippetTypeConfig {
                    source_file: file_path.clone(),
                    description: None,
                },
            );
            map
        },
        config_paths: vec![],
        active_config_path: None,
    };

    // Set a mock editor that just touches the file
    env::set_var("EDITOR", "touch");

    let cli = Cli {
        debug: 0,
        generator: None,
        generate_config: false,
        info: false,
        command: Some(Commands::Edit {
            ctype: Some("test".to_string()),
        }),
    };

    // Act
    execute_command(&cli, &config)?;

    // Assert
    assert!(file_path.exists());
    Ok(())
}

#[test]
fn given_snippet_with_comments_when_parsing_then_preserves_comments() {
    // Arrange
    let content = r#"
: File level comment (ignored)
--- apple
: This is a comment about apples
: Another comment
this is green
and nothing else
---"#;
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    writeln!(temp_file, "{}", content).expect("Failed to write test content");
    let path = temp_file.path();

    // Act
    let snippets = parse_snippets_file(path).expect("Should parse successfully");

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
}
