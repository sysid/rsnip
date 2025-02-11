use anyhow::Result;
use rsnip::domain::parser::{SnippetFormat, SnippetType};
use std::io::Write;
use tempfile::NamedTempFile;
use rsnip::application::snippet_service::SnippetService;

fn create_test_snippet_type(path: &std::path::Path) -> SnippetType {
    SnippetType {
        name: "test".to_string(),
        source_file: path.to_path_buf(),
        format: SnippetFormat::Default,
    }
}

#[test]
fn given_valid_snippet_file_when_getting_snippets_then_returns_snippets() -> Result<()> {
    // Arrange
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "--- test\nContent\n---")?;
    let snippet_type = create_test_snippet_type(temp_file.path());
    let service = SnippetService::new();

    // Act
    let snippets = service.get_snippets(&snippet_type)?;

    // Assert
    assert_eq!(snippets.len(), 1);
    assert_eq!(snippets[0].name, "test");
    assert_eq!(snippets[0].content.get_content(), "Content");

    Ok(())
}

#[test]
fn given_empty_input_when_finding_completion_then_returns_none() -> Result<()> {
    // Arrange
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "--- apple\nA red fruit\n---\n--- banana\nA yellow fruit\n---")?;
    let snippet_type = create_test_snippet_type(temp_file.path());
    let service = SnippetService::new();

    // Act & Assert - Test exact match
    assert!(service.find_completion_exact(&snippet_type, "")?.is_none());

    // Act & Assert - Test fuzzy match
    assert!(service.find_completion_fuzzy(&snippet_type, "")?.is_none());

    Ok(())
}

#[test]
fn given_exact_match_when_finding_completion_then_returns_snippet() -> Result<()> {
    // Arrange
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "--- test\nContent\n---")?;
    let snippet_type = create_test_snippet_type(temp_file.path());
    let service = SnippetService::new();

    // Act
    let result = service.find_completion_exact(&snippet_type, "test")?;

    // Assert
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "test");
    Ok(())
}

#[test]
fn given_fuzzy_match_when_finding_completion_then_returns_best_match() -> Result<()> {
    // Arrange
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "--- test\nContent\n---\n--- testing\nContent2\n---")?;
    let snippet_type = create_test_snippet_type(temp_file.path());
    let service = SnippetService::new();

    // Act
    let result = service.find_completion_fuzzy(&snippet_type, "tst")?;

    // Assert
    assert!(result.is_some());
    assert!(result.unwrap().name.contains("test"));
    Ok(())
}

#[test]
fn given_nonexistent_snippet_when_copying_then_returns_none() -> Result<()> {
    // Arrange
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "--- apple\nA red fruit\n---")?;
    let snippet_type = create_test_snippet_type(temp_file.path());
    let service = SnippetService::new();

    // Act
    let result = service.copy_snippet_to_clipboard(&snippet_type, "nonexistent", true)?;

    // Assert
    assert!(result.is_none());
    Ok(())
}

#[test]
fn given_template_snippet_when_copying_then_returns_rendered_content() -> Result<()> {
    // Arrange
    let mut temp_file = NamedTempFile::new()?;
    let template_content = "--- date\n{{current_date|strftime('%Y-%m-%d')}}\n---";
    writeln!(temp_file, "{}", template_content)?;
    let snippet_type = create_test_snippet_type(temp_file.path());
    let service = SnippetService::new();

    // Act
    let result = service.copy_snippet_to_clipboard(&snippet_type, "date", true)?;

    // Assert
    assert!(result.is_some());
    let (snippet, rendered) = result.unwrap();
    assert_eq!(snippet.name, "date");

    let is_valid_date = rendered.trim().len() == 10
        && rendered.trim().chars().all(|c| c.is_ascii_digit() || c == '-')
        && rendered.matches('-').count() == 2;

    assert!(is_valid_date, "Rendered date '{}' is not in YYYY-MM-DD format", rendered);
    Ok(())
}