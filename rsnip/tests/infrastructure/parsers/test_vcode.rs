use anyhow::Result;
use rsnip::domain::parser::SnippetParser;
use rsnip::infrastructure::parsers::VCodeSnippetParser;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn given_valid_vscode_file_when_parse_then_returns_snippets() -> Result<()> {
    // Arrange
    let content = r#"{
            "Rust Hello World": {
                "prefix": "rust-hello",
                "body": [
                    "fn main() {",
                    "    println!(\"Hello, world!\");",
                    "}"
                ],
                "description": "Insert a simple Rust Hello World program"
            },
            "Rust Function": {
                "prefix": "rust-fn",
                "body": [
                    "fn ${1:function_name}(${2:params}) -> ${3:ReturnType} {",
                    "    ${4:// function body}",
                    "}"
                ],
                "description": "Create a Rust function template"
            }
        }"#;

    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = VCodeSnippetParser::new();

    // Act
    let snippets = parser.parse(temp_file.path())?;

    // Assert
    assert_eq!(snippets.len(), 2);

    let hello_snippet = snippets.iter().find(|s| s.name == "rust-hello").unwrap();
    // assert_eq!(
    //     hello_snippet.content.get_content(),
    //     "fn main() {\n    println!(\"Hello, world!\");\n}"
    // );
    assert_eq!(
        hello_snippet.content.get_content(),
        r#"fn main() {
    println!("Hello, world!");
}"#
    );
    assert_eq!(
        hello_snippet.comments,
        vec!["Insert a simple Rust Hello World program"]
    );

    let fn_snippet = snippets.iter().find(|s| s.name == "rust-fn").unwrap();
    assert_eq!(
        fn_snippet.content.get_content(),
        "fn ${1:function_name}(${2:params}) -> ${3:ReturnType} {\n    ${4:// function body}\n}"
    );
    assert_eq!(fn_snippet.comments, vec!["Create a Rust function template"]);

    Ok(())
}

#[test]
fn given_empty_vscode_file_when_parse_then_returns_empty_vec() -> Result<()> {
    // Arrange
    let content = "{}";
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = VCodeSnippetParser::new();

    // Act
    let snippets = parser.parse(temp_file.path())?;

    // Assert
    assert!(snippets.is_empty());
    Ok(())
}

#[test]
fn given_malformed_json_when_parse_then_returns_error() -> Result<()> {
    // Arrange
    let content = r#"{ malformed json }"#;
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = VCodeSnippetParser::new();

    // Act
    let result = parser.parse(temp_file.path());

    // Assert
    assert!(result.is_err());
    Ok(())
}

#[test]
fn given_single_line_body_when_parse_then_handles_correctly() -> Result<()> {
    // Arrange
    let content = r#"{
            "Single Line": {
                "prefix": "single",
                "body": "println!(\"${1:message}\");",
                "description": "Single line snippet"
            }
        }"#;

    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "{}", content)?;
    let parser = VCodeSnippetParser::new();

    // Act
    let snippets = parser.parse(temp_file.path())?;

    // Assert
    assert_eq!(snippets.len(), 1);
    assert_eq!(
        snippets[0].content.get_content(),
        "println!(\"${1:message}\");"
    );
    assert_eq!(snippets[0].comments, vec!["Single line snippet"]);
    Ok(())
}
