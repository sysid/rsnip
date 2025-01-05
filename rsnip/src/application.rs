/// Application logic for the rsnip tool: Use Cases
use crate::{
    domain::{CompletionItem, CompletionType},
    infrastructure,
};
use anyhow::{Context, Result};
use skim::{prelude::*, Skim};
use std::{io::Cursor, cmp::Reverse};
use fuzzy_matcher::skim::SkimMatcherV2;

const FUZZY_FINDER_HEIGHT: &str = "50%";

/// Finds completions interactively using fuzzy finder
pub fn find_completion_interactive(
    completion_type: &CompletionType,
    user_input: &str,
) -> Result<Option<CompletionItem>> {
    // Read and cache items
    let items = infrastructure::read_completions_from_file(&completion_type.source_file)
        .context("Failed to read completion source file")?;

    // Early return if no items
    if items.is_empty() {
        return Ok(None);
    }

    // Format items for display once
    let display_items = format_items_for_display(&items);

    let selected_item = run_fuzzy_finder(&display_items, user_input)?;

    // Map selected text back to original item
    Ok(selected_item.and_then(|text| {
        let clean_text = text.split('\t').next().unwrap_or(&text);
        items.iter().find(|item| item.text == clean_text).cloned()
    }))
}

/// Find first matching completion non-interactively
pub fn find_completion(
    completion_type: &CompletionType,
    user_input: &str,
) -> Result<Option<CompletionItem>> {
    // Return None for empty input
    if user_input.trim().is_empty() {
        return Ok(None);
    }

    let items = infrastructure::read_completions_from_file(&completion_type.source_file)?;
    let matcher = SkimMatcherV2::default();

    // Find best match using iterator
    Ok(items.into_iter()
        .filter_map(|item| {
            matcher.fuzzy(&item.text, user_input, false)
                .map(|score| (Reverse(score), item))  // Reverse for max_by_key
        })
        .max_by_key(|(score, _)| score.clone())
        .map(|(_, item)| item))
}

// Helper Functions

fn format_items_for_display(items: &[CompletionItem]) -> Vec<String> {
    items.iter()
        .map(|item| {
            item.description.as_ref()
                .map_or_else(
                    || item.text.clone(),
                    |desc| format!("{}\t{}", item.text, desc)
                )
        })
        .collect()
}

fn run_fuzzy_finder(items: &[String], initial_query: &str) -> Result<Option<String>> {
    let options = SkimOptionsBuilder::default()
        .height(FUZZY_FINDER_HEIGHT.to_string())
        .multi(false)
        .preview(Some("".to_string()))  // Empty preview window
        .bind(vec!["Enter:accept".to_string()])
        .query(Some(initial_query.to_string()))
        .select1(true)  // Auto-select if only one match
        .build()?;

    let input = items.join("\n");
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    let selected = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_default();

    Ok(selected.first().map(|item| item.output().to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use crate::domain::CompletionAction;

    #[test]
    fn given_empty_input_when_finding_completion_then_returns_none() {
        let mut tmp = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(tmp, "apple|A red fruit\nbanana|A yellow fruit").unwrap();

        let ctype = CompletionType {
            name: "test".to_string(),
            source_file: tmp.path().into(),
            keyboard_shortcut: "ctrl+t".to_string(),
            action: CompletionAction::CopyToClipboard,
        };

        let result = find_completion(&ctype, "");
        assert!(result.unwrap().is_none());
    }
}