// application/services/tests/management_tests.rs
use anyhow::Result;
use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use rsnip::application::services::SnippetManagementService;
use rsnip::application::snippet_service::SnippetService;
use rsnip::config::{Settings, SnippetTypeConfig};
use rsnip::infrastructure::minijinja::{MiniJinjaEngine, SafeShellExecutor};

fn create_test_settings(files: &[(String, PathBuf)]) -> Settings {
    let mut snippet_types = HashMap::new();

    // Add concrete types
    for (name, path) in files {
        snippet_types.insert(
            name.clone(),
            SnippetTypeConfig::Concrete {
                source_file: path.clone(),
                description: None,
                alias: None,
                format: "default".to_string(),
            },
        );
    }

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

    let settings = create_test_settings(&[("test".to_string(), temp_file.path().to_path_buf())]);
    let service = SnippetManagementService::new(&settings);

    // Act
    let snippets = service.get_snippets("test")?;

    // Assert
    assert_eq!(snippets.len(), 1);
    assert_eq!(snippets[0].name, "test");
    assert_eq!(snippets[0].content.get_content(), "Content");
    Ok(())
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
    let template_engine = Box::new(MiniJinjaEngine::new(Box::new(SafeShellExecutor::new())));
    let service = SnippetService::new(template_engine, &settings);

    // Act
    let snippets = service.get_snippets("combined")?;

    // Assert
    assert_eq!(snippets.len(), 2);
    assert!(snippets.iter().any(|s| s.name == "test1"));
    assert!(snippets.iter().any(|s| s.name == "test2"));
    Ok(())
}