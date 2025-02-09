use rsnip::domain::{Snippet, SnippetContent, SnippetType};
use rsnip::fuzzy::{create_skim_items, run_fuzzy_finder};
use skim::{ItemPreview, PreviewContext};
use std::path::PathBuf;

fn create_test_snippets() -> Vec<Snippet> {
    vec![
        Snippet {
            name: "apple".to_string(),
            content: SnippetContent::Static("This is an apple".to_string()),
            comments: Vec::new(), // Add empty comments vector
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
    ]
}

fn create_test_snippet_type() -> SnippetType {
    SnippetType {
        name: "test".to_string(),
        source_file: PathBuf::from("test_snippets.txt"),
    }
}

#[test]
fn given_exact_match_when_fuzzy_finder_then_returns_immediately() {
    let items = create_test_snippets();
    let snippet_type = create_test_snippet_type();
    let result = run_fuzzy_finder(&items, &snippet_type, "apple").unwrap();
    assert_eq!(result, Some("apple".to_string()));
}

#[test]
fn given_single_partial_match_when_fuzzy_finder_then_auto_selects() {
    let items = create_test_snippets();
    let snippet_type = create_test_snippet_type();
    let result = run_fuzzy_finder(&items, &snippet_type, "ban").unwrap();
    assert_eq!(result, Some("banana".to_string()));
}

#[test]
#[ignore = "This test is interactive via Makefile"]
fn given_no_matches_when_fuzzy_finder_then_shows_interface() {
    let items = create_test_snippets();
    let snippet_type = create_test_snippet_type();

    // Use a query that won't match any items
    let result = run_fuzzy_finder(&items, &snippet_type, "xyz").unwrap();

    // The result should be None since we can't simulate user interaction in tests,
    // but the function should have attempted to show the interface rather than
    // returning early
    assert!(result.is_none());
}

// #[test]
// fn given_multiple_matches_when_fuzzy_finder_then_shows_interface() {
//     let items = create_test_snippets();
//     // This will open the fuzzy finder interface when run manually
//     // For automated testing, we can only verify it doesn't return immediately
//     let result = run_fuzzy_finder(&items, "ap").unwrap();
//     assert!(
//         result.is_none()
//             || result == Some("apple".to_string())
//             || result == Some("apricot".to_string())
//     );
// }

// #[test]
// fn given_empty_query_when_fuzzy_finder_then_shows_all() {
//     let items = create_test_snippets();
//     let result = run_fuzzy_finder(&items, "").unwrap();
//     // Should open interface showing all items
//     assert!(result.is_none() || items.iter().any(|item| Some(item.name.clone()) == result));
// }

#[test]
fn given_empty_items_when_fuzzy_finder_then_returns_none() {
    let items: Vec<Snippet> = vec![];
    let snippet_type = create_test_snippet_type();
    let result = run_fuzzy_finder(&items, &snippet_type, "anything").unwrap();
    assert_eq!(result, None);
}

#[test]
fn given_snippets_when_creating_skim_items_then_returns_formatted_items() {
    // Arrange
    let snippets = vec![
        Snippet {
            name: "test1".to_string(),
            content: SnippetContent::Static("line1\nline2\nline3".to_string()),
            comments: Vec::new(),
        },
        Snippet {
            name: "test2".to_string(),
            content: SnippetContent::Static("".to_string()),
            comments: Vec::new(),
        },
    ];

    let snippet_type = SnippetType {
        name: "test".to_string(),
        source_file: PathBuf::from("test.txt"),
    };

    // Act
    let items = create_skim_items(&snippets, &snippet_type);

    // Assert
    // Check that we get the correct number of items
    assert_eq!(items.len(), 2);

    // Test output (name) for both items
    assert_eq!(items[0].text(), "test1");
    assert_eq!(items[1].text(), "test2");

    // Create a preview context
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

    // Test preview content
    if let ItemPreview::AnsiText(preview) = items[0].preview(preview_context) {
        assert!(preview.contains("line1\nline2\nline3"));
        assert!(preview.contains("Name"));
        assert!(preview.contains("Content"));
    } else {
        panic!("Expected AnsiText preview");
    }

    // Create a preview context
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
    } else {
        panic!("Expected AnsiText preview");
    }
}

#[test]
#[ignore = "does not work in IDE"]
fn test_fuzzy_finder_output_is_clean() {
    let items = create_test_snippets();
    let snippet_type = SnippetType {
        name: "test".to_string(),
        source_file: PathBuf::from("test.txt"),
    };

    let result = run_fuzzy_finder(&items, &snippet_type, "test").unwrap();
    // Verify no ANSI escape sequences in output
    assert!(result.as_ref().map_or(true, |s| !s.contains("\x1B[")));
}
