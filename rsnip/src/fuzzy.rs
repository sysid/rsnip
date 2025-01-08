use crate::domain::{Snippet, SnippetContent};
use crate::domain::SnippetType;
use anyhow::Result;
use crossterm::execute;
use crossterm::terminal::{Clear, ClearType};
use fuzzy_matcher::skim::SkimMatcherV2;
use skim::{prelude::*, Skim};
use std::sync::Arc;
use tracing::{debug, trace};

// Struct to hold snippet text, preview and source info
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
pub fn create_skim_items(items: &[Snippet], _: &SnippetType) -> Vec<Arc<dyn SkimItem>> {
    let mut snippet_items = Vec::new();

    for item in items {
        // Get content as string
        let content_str = match &item.content {
            SnippetContent::Static(s) => s,
            SnippetContent::Template { source, .. } => source,
        };

        // For display text, show name and first line of content if available
        let display_text = if content_str.is_empty() {
            item.name.clone()
        } else {
            format!("{}\t{}", item.name, content_str.lines().next().unwrap_or(""))
        };

        // For preview, show full content
        let preview = format!(
            "Name: {}\nContent:\n{}",
            item.name,
            if content_str.is_empty() { "No content" } else { content_str }
        );

        snippet_items.push(Arc::new(SnippetItem {
            display_text,
            preview,
        }) as Arc<dyn SkimItem>);
    }

    snippet_items
}

/// Run fuzzy finder with multiline preview support and edit capability
pub fn run_fuzzy_finder(
    items: &[Snippet],
    snippet_type: &SnippetType,
    initial_query: &str,
) -> Result<Option<String>> {
    debug!("Starting fuzzy finder with query: {}", initial_query);

    // Early return if no items
    if items.is_empty() {
        debug!("No items to search through");
        return Ok(None);
    }

    // Early exact match check
    if !initial_query.is_empty() {
        if let Some(exact) = items.iter().find(|i| i.name == initial_query) {
            debug!("Found exact match: {}", exact.name);
            return Ok(Some(exact.name.clone()));
        }
    }

// If we have a non-empty query, check for exact match first
    if !initial_query.is_empty() {
        // Check for exact match first
        if let Some(exact) = items.iter().find(|i| i.name == initial_query) {
            debug!("Found exact match: {}", exact.name);
            return Ok(Some(exact.name.clone()));
        }

        let matcher = SkimMatcherV2::default();
        // Then collect all fuzzy matches
        let matches: Vec<_> = items
            .iter()
            .filter(|item| matcher.fuzzy(&item.name, initial_query, false).is_some())
            .collect();

        debug!("Found {} fuzzy matches", matches.len());

        // If exactly one fuzzy match, return it
        if matches.len() == 1 {
            debug!("Single fuzzy match found: {}", matches[0].name);
            return Ok(Some(matches[0].name.clone()));
        }

        // For no matches or multiple matches, continue to fuzzy finder UI
        debug!("Launching UI with {} matches", matches.len());
    }

    let options = SkimOptionsBuilder::default()
        .height("50%".to_string())
        .multi(false)
        .preview(Some("right:50%".to_string()))
        .bind(vec![
            "ctrl-e:accept".to_string(),
            "enter:accept".to_string(),
        ])
        // These three options are key for auto-selection:
        .filter(Some(initial_query.to_string())) // Immediately apply filter
        .select_1(true) // Auto-select if single match
        .exit_0(true) // Exit if no matches
        .build()?;

    let skim_items = create_skim_items(items, snippet_type);
    trace!("Items created: {}", skim_items.len());

    // Use unbounded channel to prevent potential deadlock
    let (tx_sink, rx_reader): (SkimItemSender, SkimItemReceiver) = unbounded();
    for item in skim_items {
        tx_sink.send(item)?;
    }
    drop(tx_sink); // Close sender after all items sent

    let mut stderr = std::io::stderr();  // this is the key for proper terminal cleanup
    let selected = Skim::run_with(&options, Some(rx_reader))
        .map(|out| {
            // Always clean up terminal state after Skim closes
            execute!(stderr, Clear(ClearType::FromCursorUp)).ok();

            match out.final_key {
                Key::Ctrl('e') => {
                    let selected_items = out.selected_items.iter()
                        .filter_map(|selected_item| {
                            (**selected_item)
                                .as_any()
                                .downcast_ref::<SnippetItem>()
                                .map(|item| item.to_owned())
                        })
                        .collect::<Vec<SnippetItem>>();

                    if let Some(item) = selected_items.first() {
                        debug!("Opening editor for: {}", item.display_text);
                    }
                    None
                }
                Key::Enter => {
                    out.selected_items
                        .first()
                        .map(|item| item.output().to_string())
                }
                _ => None,
            }
        })
        .unwrap_or_default();

    // Add additional terminal cleanup just in case
    // execute!(stdout, Clear(ClearType::FromCursorDown)).ok();
    Ok(selected.map(|s| s.split('\t').next().unwrap_or("").to_string()))
}
