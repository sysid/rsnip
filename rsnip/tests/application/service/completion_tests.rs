// application/services/tests/completion_tests.rs

use rsnip::application::services::CompletionService;
use rsnip::application::snippet_service::SnippetService;
use rsnip::config::{Settings, SnippetTypeConfig};
use rsnip::domain::content::SnippetContent;
use rsnip::domain::snippet::Snippet;
use rsnip::infrastructure::minijinja::{MiniJinjaEngine, SafeShellExecutor};
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_test_snippets() -> Vec<Snippet> {
    vec![
        Snippet {
            name: "apple".to_string(),
            content: SnippetContent::Static("apple content".to_string()),
            comments: vec![],
        },
        Snippet {
            name: "banana".to_string(),
            content: SnippetContent::Static("banana content".to_string()),
            comments: vec![],
        },
    ]
}

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
fn given_exact_match_when_finding_completion_then_returns_snippet() {
    // Arrange
    let service = CompletionService::new();
    let snippets = create_test_snippets();

    // Act
    let result = service.find_completion_exact(&snippets, "apple");

    // Assert
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "apple");
}

#[test]
fn given_fuzzy_match_when_finding_completion_then_returns_best_match() {
    // Arrange
    let service = CompletionService::new();
    let snippets = create_test_snippets();

    // Act
    let result = service.find_completion_fuzzy(&snippets, "apl");

    // Assert
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "apple");
}

#[test]
fn given_empty_input_when_finding_completion_then_returns_none() -> anyhow::Result<()> {
    // Arrange
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "--- apple\nA red fruit\n---\n--- banana\nA yellow fruit\n---")?;
    let settings = create_test_settings_single(temp_file.path().to_path_buf());
    let template_engine = Box::new(MiniJinjaEngine::new(Box::new(SafeShellExecutor::new())));
    let service = SnippetService::new(template_engine, &settings);

    // Act & Assert - Test exact match
    assert!(service.find_completion_exact("test", "")?.is_none());

    // Act & Assert - Test fuzzy match
    assert!(service.find_completion_fuzzy("test", "")?.is_none());

    Ok(())
}