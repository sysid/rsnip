use rsnip::infrastructure::fuzzy::{create_skim_items, run_fuzzy_finder};
use skim::{ItemPreview, PreviewContext};
use std::path::PathBuf;
use rsnip::domain::content::SnippetContent;
use rsnip::domain::parser::{SnippetFormat, SnippetType};
use rsnip::domain::snippet::Snippet;
use anyhow::Result;

// Helper function to create consistent test data
fn create_test_data() -> (Vec<Snippet>, SnippetType) {
    let snippets = vec![
        Snippet {
            name: "apple".to_string(),
            content: SnippetContent::Static("This is an apple".to_string()),
            comments: Vec::new(),
        },
        Snippet {
            name: "apricot".to_string(),
            content: SnippetContent::Static("This is an apricot".to_string()),
            comments: Vec::new(),
        },
        Snippet {
            name: "banana".to_string(),
            content: SnippetContent::Static("This is a banana".to_string()),
            comments: Vec::new(),
        },
    ];

    let snippet_type = SnippetType {
        name: "test".to_string(),
        source_file: PathBuf::from("test_snippets.txt"),
        format: SnippetFormat::Default,
    };

    (snippets, snippet_type)
}

#[test]
fn given_exact_match_when_fuzzy_finder_then_returns_immediately() -> Result<()> {
    let (items, _) = create_test_data();
    let result = run_fuzzy_finder(&items, "apple")?;
    assert_eq!(result, Some("apple".to_string()));
    Ok(())
}

#[test]
fn given_single_partial_match_when_fuzzy_finder_then_auto_selects() -> Result<()> {
    let (items, _) = create_test_data();
    let result = run_fuzzy_finder(&items, "ban")?;
    assert_eq!(result, Some("banana".to_string()));
    Ok(())
}

#[test]
#[ignore = "This test is interactive via Makefile"]
fn given_multiple_matches_when_fuzzy_finder_then_shows_interface() -> Result<()> {
    let (items, _) = create_test_data();
    let result = run_fuzzy_finder(&items, "ap")?;

    // Since we can't simulate user interaction in unit tests,
    // we can only verify it doesn't return an invalid result
    if let Some(name) = result {
        assert!(name == "apple" || name == "apricot",
            "Expected either 'apple' or 'apricot', got '{}'", name);
    }
    Ok(())
}

#[test]
#[ignore = "todo fix"]
fn given_empty_query_when_fuzzy_finder_then_shows_all() -> Result<()> {
    let (items, _) = create_test_data();
    let result = run_fuzzy_finder(&items, "")?;

    // For empty query, it should either:
    // 1. Return None (if user cancels)
    // 2. Return a valid snippet name (if user selects one)
    if let Some(name) = result {
        assert!(
            items.iter().any(|item| item.name == name),
            "Returned name '{}' not found in test items", name
        );
    }
    Ok(())
}

#[test]
fn given_empty_items_when_fuzzy_finder_then_returns_none() -> Result<()> {
    let items: Vec<Snippet> = vec![];
    let result = run_fuzzy_finder(&items, "anything")?;
    assert_eq!(result, None);
    Ok(())
}

#[test]
fn given_snippets_when_creating_skim_items_then_returns_formatted_items() {
    // Arrange
    let snippets = vec![
        Snippet {
            name: "test1".to_string(),
            content: SnippetContent::Static("line1\nline2\nline3".to_string()),
            comments: vec!["A test comment".to_string()],
        },
        Snippet {
            name: "test2".to_string(),
            content: SnippetContent::Static("".to_string()),
            comments: Vec::new(),
        },
    ];

    // Act
    let items = create_skim_items(&snippets);

    // Assert
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].text(), "test1");
    assert_eq!(items[1].text(), "test2");

    // Test preview with content and comments
    let preview_context = PreviewContext {
        query: "",
        cmd_query: "",
        width: 0,
        height: 0,
        current_index: 0,
        current_selection: "",
        selected_indices: &[],
        selections: &[],
    };

    if let ItemPreview::AnsiText(preview) = items[0].preview(preview_context) {
        assert!(preview.contains("line1\nline2\nline3"));
        assert!(preview.contains("Name"));
        assert!(preview.contains("Content"));
        assert!(preview.contains("Comments"));
        assert!(preview.contains("A test comment"));
    } else {
        panic!("Expected AnsiText preview");
    }

    // Test preview with empty content
    let preview_context = PreviewContext {
        query: "",
        cmd_query: "",
        width: 0,
        height: 0,
        current_index: 0,
        current_selection: "",
        selected_indices: &[],
        selections: &[],
    };

    if let ItemPreview::AnsiText(preview) = items[1].preview(preview_context) {
        assert!(preview.contains("No content"));
        assert!(preview.contains("Name"));
        assert!(preview.contains("Content"));
        assert!(!preview.contains("Comments")); // No comments section for empty comments
    } else {
        panic!("Expected AnsiText preview");
    }
}

#[test]
#[ignore = "Interactive test requires terminal"]
fn given_no_matches_when_fuzzy_finder_then_shows_interface() -> Result<()> {
    let (items, _) = create_test_data();
    let result = run_fuzzy_finder(&items, "xyz")?;
    assert!(result.is_none());
    Ok(())
}

#[test]
#[ignore = "Interactive test requires terminal"]
fn test_fuzzy_finder_output_is_clean() -> Result<()> {
    let (items, _) = create_test_data();
    let result = run_fuzzy_finder(&items, "test")?;
    assert!(result.as_ref().map_or(true, |s| !s.contains("\x1B[")));
    Ok(())
}