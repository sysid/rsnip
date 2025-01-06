use rsnip::domain::Snippet;
use rsnip::fuzzy::run_fuzzy_finder;

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

#[test]
fn given_exact_match_when_fuzzy_finder_then_returns_immediately() {
    let items = create_test_snippets();
    let result = run_fuzzy_finder(&items, "apple").unwrap();
    assert_eq!(result, Some("apple".to_string()));
}

#[test]
fn given_single_partial_match_when_fuzzy_finder_then_auto_selects() {
    let items = create_test_snippets();
    let result = run_fuzzy_finder(&items, "ban").unwrap();
    assert_eq!(result, Some("banana".to_string()));
}

#[test]
fn given_no_matches_when_fuzzy_finder_then_returns_none() {
    let items = create_test_snippets();
    let result = run_fuzzy_finder(&items, "xyz").unwrap();
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
    let result = run_fuzzy_finder(&items, "anything").unwrap();
    assert_eq!(result, None);
}
