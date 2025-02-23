use anyhow::Result;
use rsnip::config::{Settings, SnippetTypeConfig};
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;
use rsnip::application::snippet_service::SnippetService;
use rsnip::infrastructure::minijinja::{MiniJinjaEngine, SafeShellExecutor};

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

#[test]
fn given_nonexistent_snippet_when_copying_then_returns_none() -> Result<()> {
    // Arrange
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "--- apple\nA red fruit\n---")?;
    let settings = create_test_settings_single(temp_file.path().to_path_buf());
    let template_engine = Box::new(MiniJinjaEngine::new(Box::new(SafeShellExecutor::new())));
    let service = SnippetService::new(template_engine, &settings);

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
    let template_engine = Box::new(MiniJinjaEngine::new(Box::new(SafeShellExecutor::new())));
    let service = SnippetService::new(template_engine, &settings);

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

