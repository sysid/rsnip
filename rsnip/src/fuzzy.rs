use crate::domain::Snippet;
use anyhow::Result;
use skim::{prelude::*, Skim};
use tracing::trace;

// Struct to hold snippet text and preview
#[derive(Clone)]
struct SnippetItem {
    display_text: String,
    preview: String,
}

impl SkimItem for SnippetItem {
    fn text(&self) -> Cow<str> {
        Cow::Borrowed(&self.display_text)
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        ItemPreview::Text(self.preview.clone())
    }

    fn output(&self) -> Cow<str> {
        self.text()
    }
}

/// Format snippets for display and preview
fn create_skim_items(items: &[Snippet]) -> Vec<Arc<dyn SkimItem>> {
    items
        .iter()
        .map(|item| {
            // For display text, show name and first line of snippet if available
            let display_text = item.snippet.as_ref().map_or_else(
                || item.name.clone(),
                |desc| format!("{}\t{}", item.name, desc.lines().next().unwrap_or("")),
            );

            // For preview, show full snippet content
            let preview = format!(
                "Name: {}\nContent:\n{}",
                item.name,
                item.snippet
                    .as_ref()
                    .map_or_else(|| String::from("No content"), |content| content.clone())
            );

            Arc::new(SnippetItem {
                display_text,
                preview,
            }) as Arc<dyn SkimItem>
        })
        .collect()
}

/// Run fuzzy finder with multiline preview support
pub fn run_fuzzy_finder(items: &[Snippet], initial_query: &str) -> Result<Option<String>> {
    trace!("Starting fuzzy finder"); // Debug

    if let Some(exact) = items.iter().find(|i| i.name == initial_query) {
        return Ok(Some(exact.name.clone()));
    }

    let options = SkimOptionsBuilder::default()
        .height("50%".to_string())
        .multi(false)
        .preview(Some("right:50%".to_string()))
        .bind(vec!["Enter:accept".to_string()])
        .query(Some(initial_query.to_string()))
        .select_1(true)
        .exit_0(true)
        .build()?;

    trace!("Options built"); // Debug

    let skim_items = create_skim_items(items);
    trace!("Items created: {}", skim_items.len()); // Add item count debug

    // Use unbounded channel to prevent potential deadlock
    let (tx_sink, rx_reader): (SkimItemSender, SkimItemReceiver) = unbounded();

    trace!("Channel created"); // Debug

    for item in skim_items {
        tx_sink.send(item)?;
    }
    // Close the sender so skim knows we're done
    drop(tx_sink);

    trace!("Items sent"); // Debug

    let selected = Skim::run_with(&options, Some(rx_reader))
        .map(|out| out.selected_items)
        .unwrap_or_default();

    trace!("Skim completed"); // Debug

    Ok(selected.first().map(|item| {
        // Extract just the name part before the tab character
        item.output().split('\t').next().unwrap_or("").to_string()
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Snippet;

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

        let items = create_skim_items(&snippets);

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
}
