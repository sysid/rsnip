use crate::domain::{SnippetType, Snippet};
use anyhow::{Context, Result};
use skim::{prelude::*, Skim};
use std::{cmp::Reverse, io::Cursor};
use fuzzy_matcher::skim::SkimMatcherV2;
use crate::infrastructure;
use crate::infrastructure::parse_snippets_file;

const FUZZY_FINDER_HEIGHT: &str = "50%";

/// Finds completions interactively using fuzzy finder
pub fn find_completion_interactive(
    completion_type: &SnippetType,
    user_input: &str,
) -> Result<Option<Snippet>> {
    // Read and parse snippets
    let items = parse_snippets_file(&completion_type.source_file)
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
        items.iter().find(|item| item.name == clean_text).cloned()
    }))
}

/// Find first matching completion non-interactively
pub fn find_completion(
    completion_type: &SnippetType,
    user_input: &str,
) -> Result<Option<Snippet>> {
    // Return None for empty input
    if user_input.trim().is_empty() {
        return Ok(None);
    }

    let items = parse_snippets_file(&completion_type.source_file)?;
    let matcher = SkimMatcherV2::default();

    // Find best match using iterator
    Ok(items.into_iter()
        .filter_map(|item| {
            matcher.fuzzy(&item.name, user_input, false)
                .map(|score| (Reverse(score), item))  // Reverse for max_by_key
        })
        .max_by_key(|(score, _)| score.clone())
        .map(|(_, item)| item))
}

// Helper Functions

fn format_items_for_display(items: &[Snippet]) -> Vec<String> {
    items.iter()
        .map(|item| {
            item.snippet.as_ref()
                .map_or_else(
                    || item.name.clone(),
                    |desc| format!("{}\t{}", item.name, desc.lines().next().unwrap_or(""))
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
        .select_1(true)    // Auto-select if only one match
        .exit_0(true)      // Exit immediately when there's no match
        .build()?;

    let input = items.join("\n");
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    let selected = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_default();

    Ok(selected.first().map(|item| item.output().to_string()))
}

pub fn copy_snippet_to_clipboard(
    completion_type: &SnippetType,
    input: &str,
) -> Result<Option<Snippet>> {
    if let Some(completion_item) = find_completion(completion_type, input)? {
        if let Some(snippet) = &completion_item.snippet {
            infrastructure::copy_to_clipboard(snippet)?;
        }
        Ok(Some(completion_item))
    } else {
        Ok(None)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn given_empty_input_when_finding_completion_then_returns_none() -> Result<()> {
        let mut tmp = NamedTempFile::new()?;
        writeln!(tmp, "--- apple\nA red fruit\n---\n--- banana\nA yellow fruit\n---")?;

        let ctype = SnippetType {
            name: "test".to_string(),
            source_file: tmp.path().into(),
        };

        assert!(find_completion(&ctype, "")?.is_none());
        Ok(())
    }
}