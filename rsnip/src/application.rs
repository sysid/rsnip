use crate::domain::{Snippet, SnippetType};
use crate::{fuzzy, infrastructure};
use crate::infrastructure::parse_snippets_file;
use anyhow::{Context, Result};
use fuzzy_matcher::skim::SkimMatcherV2;
use std::{cmp::Reverse};
use tracing::instrument;

/// Finds completions interactively using fuzzy finder
#[instrument(level = "debug")]
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

    let selected_item = fuzzy::run_fuzzy_finder(&items, completion_type, user_input)?;

    // Return the selected snippet
    Ok(selected_item.and_then(|name| {
        items.iter().find(|item| item.name == name).cloned()
    }))
}

/// Find first matching completion non-interactively
#[instrument(level = "debug")]
pub fn find_completion_fuzzy(
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
    Ok(items
        .into_iter()
        .filter_map(|item| {
            matcher
                .fuzzy(&item.name, user_input, false)
                .map(|score| (Reverse(score), item)) // Reverse for max_by_key
        })
        .max_by_key(|(score, _)| score.clone())
        .map(|(_, item)| item))
}

/// Find a completion using an exact match
#[instrument(level = "debug")]
pub fn find_completion_exact(
    completion_type: &SnippetType,
    user_input: &str,
) -> Result<Option<Snippet>> {
    if user_input.trim().is_empty() {
        return Ok(None);
    }

    let items = parse_snippets_file(&completion_type.source_file)?;
    Ok(items.into_iter().find(|item| item.name == user_input))
}

#[instrument(level = "debug")]
pub fn copy_snippet_to_clipboard(
    completion_type: &SnippetType,
    input: &str,
    exact: bool,
) -> Result<Option<Snippet>> {
    let item = if exact {
        find_completion_exact(completion_type, input)?
    } else {
        find_completion_fuzzy(completion_type, input)?
    };

    if let Some(completion_item) = item {
        if let Some(snippet) = &completion_item.snippet {
            infrastructure::copy_to_clipboard(snippet)?;
        }
        Ok(Some(completion_item))
    } else {
        Ok(None)
    }
}
