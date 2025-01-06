use std::path::PathBuf;
use skim::{ItemPreview, PreviewContext};
use rsnip::domain::{Snippet, SnippetType};
use rsnip::fuzzy::{create_skim_items, run_fuzzy_finder};

fn create_test_snippets() -> Vec<Snippet> {
    vec![
        Snippet {
            name: "apple".to_string(),
            snippet: Some("This is an apple".to_string()),
        },
        Snippet {
            name: "apricot".to_string(),
            snippet: Some("This is an apricot".to_string()),
        },
        Snippet {
            name: "banana".to_string(),
            snippet: Some("This is a banana".to_string()),
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
fn given_no_matches_when_fuzzy_finder_then_returns_none() {
    let items = create_test_snippets();
    let snippet_type = create_test_snippet_type();
    let result = run_fuzzy_finder(&items, &snippet_type, "xyz").unwrap();
    assert_eq!(result, None);
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
fn test_create_skim_items() {
    let snippets = vec![
        Snippet {
            name: "test1".to_string(),
            snippet: Some("line1\nline2\nline3".to_string()),
        },
        Snippet {
            name: "test2".to_string(),
            snippet: None,
        },
    ];

    let snippet_type = create_test_snippet_type();
    let items = create_skim_items(&snippets, &snippet_type);

    assert_eq!(items[0].output(), "test1\tline1");
    assert_eq!(items[1].output(), "test2");

    // Create a preview context for testing
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

    if let ItemPreview::Text(preview) = items[0].preview(preview_context) {
        assert!(preview.contains("line1\nline2\nline3"));
    } else {
        panic!("Expected Text preview");
    }

    // Create a preview context for testing
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

    if let ItemPreview::Text(preview) = items[1].preview(preview_context) {
        assert!(preview.contains("No content"));
    } else {
        panic!("Expected Text preview");
    }
}
