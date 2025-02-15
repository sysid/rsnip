use anyhow::Result;
use rsnip::config::{Settings, SnippetTypeConfig};
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;
use rsnip::application::snippet_service::SnippetService;

fn create_test_settings_single(source_file: std::path::PathBuf) -> Settings {
    let mut snippet_types = HashMap::new();
    snippet_types.insert(
        "test".to_string(),
        SnippetTypeConfig::Concrete {
            source_file,
            description: None,
            alias: None,
            format: "default".to_string(),
        },
    );

    Settings {
        snippet_types,
        config_paths: vec![],
        active_config_path: None,
    }
}

fn create_test_settings_combined(source_files: Vec<std::path::PathBuf>) -> Settings {
    let mut snippet_types = HashMap::new();

    // Add source types
    for (idx, path) in source_files.iter().enumerate() {
        snippet_types.insert(
            format!("source{}", idx + 1),
            SnippetTypeConfig::Concrete {
                source_file: path.clone(),
                description: None,
                alias: None,
                format: "default".to_string(),
            },
        );
    }

    // Add combined type
    let sources = (1..=source_files.len())
        .map(|i| format!("source{}", i))
        .collect();

    snippet_types.insert(
        "combined".to_string(),
        SnippetTypeConfig::Combined {
            sources,
            description: None,
            alias: None,
        },
    );

    Settings {
        snippet_types,
        config_paths: vec![],
        active_config_path: None,
    }
}

#[test]
fn given_valid_snippet_file_when_getting_snippets_then_returns_snippets() -> Result<()> {
    // Arrange
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "--- test\nContent\n---")?;
    let settings = create_test_settings_single(temp_file.path().to_path_buf());
    let service = SnippetService::new(&settings);

    // Act
    let snippets = service.get_snippets("test")?;

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
    let settings = create_test_settings_single(temp_file.path().to_path_buf());
    let service = SnippetService::new(&settings);

    // Act & Assert - Test exact match
    assert!(service.find_completion_exact("test", "")?.is_none());

    // Act & Assert - Test fuzzy match
    assert!(service.find_completion_fuzzy("test", "")?.is_none());

    Ok(())
}

#[test]
fn given_exact_match_when_finding_completion_then_returns_snippet() -> Result<()> {
    // Arrange
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "--- test\nContent\n---")?;
    let settings = create_test_settings_single(temp_file.path().to_path_buf());
    let service = SnippetService::new(&settings);

    // Act
    let result = service.find_completion_exact("test", "test")?;

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
    let settings = create_test_settings_single(temp_file.path().to_path_buf());
    let service = SnippetService::new(&settings);

    // Act
    let result = service.find_completion_fuzzy("test", "tst")?;

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
    let settings = create_test_settings_single(temp_file.path().to_path_buf());
    let service = SnippetService::new(&settings);

    // Act
    let result = service.copy_snippet_to_clipboard("test", "nonexistent", true)?;

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
    let settings = create_test_settings_single(temp_file.path().to_path_buf());
    let service = SnippetService::new(&settings);

    // Act
    let result = service.copy_snippet_to_clipboard("test", "date", true)?;

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

#[test]
fn given_combined_type_when_getting_snippets_then_returns_all_snippets() -> Result<()> {
    // Arrange
    let mut temp_file1 = NamedTempFile::new()?;
    let mut temp_file2 = NamedTempFile::new()?;

    writeln!(temp_file1, "--- test1\nContent1\n---")?;
    writeln!(temp_file2, "--- test2\nContent2\n---")?;

    let files = vec![
        temp_file1.path().to_path_buf(),
        temp_file2.path().to_path_buf(),
    ];
    let settings = create_test_settings_combined(files);
    let service = SnippetService::new(&settings);

    // Act
    let snippets = service.get_snippets("combined")?;

    // Assert
    assert_eq!(snippets.len(), 2);
    assert!(snippets.iter().any(|s| s.name == "test1"));
    assert!(snippets.iter().any(|s| s.name == "test2"));
    Ok(())
}