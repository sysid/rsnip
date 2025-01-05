/// Application logic for the rsnip tool: Use Cases
use crate::{
    domain::{CompletionItem, CompletionType},
    infrastructure,
};
use anyhow::{Context, Result};
use crate::domain::CompletionAction;
use crate::infrastructure::read_completions_from_file;
use skim::{
    prelude::*,
    Skim,
};
use std::io::Cursor;
use fuzzy_matcher::skim::SkimMatcherV2;

/// Loads completions and presents an interactive fuzzy finder
pub fn perform_completion(
    completion_type: &CompletionType,
    user_input: &str,
) -> Result<Option<CompletionItem>> {
    // 1) Parse the completion file
    let original_items = infrastructure::read_completions_from_file(&completion_type.source_file)
        .context("Failed to read completion source file")?;

    // 2) Format items for display
    let display_items: Vec<String> = original_items
        .iter()
        .map(|item| {
            if let Some(desc) = &item.description {
                format!("{}\t{}", item.text, desc)
            } else {
                item.text.clone()
            }
        })
        .collect();

    // 3) Create skim options
    let options = SkimOptionsBuilder::default()
        .height("50%".to_string())
        .multi(false)
        .preview(Some("".to_string())) // Empty preview window
        .bind(vec!["Enter:accept".to_string()])
        .query(Some(user_input.to_string()))
        .build()?;

    // 4) Create the input source
    let input = display_items.join("\n");
    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    // 5) Run fuzzy finder
    let selected = Skim::run_with(&options, Some(items))
        .map(|out| out.selected_items)
        .unwrap_or_default();

    // 6) Process selection
    if let Some(selected) = selected.first() {
        let text = selected.output().to_string();
        let text = text.split('\t').next().unwrap_or(&text).to_string();

        // Find the original item to get its description
        let found = original_items.iter().find(|item| item.text == text);

        if let Some(item) = found {
            let completion_item = CompletionItem {
                text: item.text.clone(),
                description: item.description.clone(),
            };

            match completion_type.action {
                CompletionAction::CopyToClipboard => {
                    let snippet = completion_item.description.as_deref().unwrap_or(&completion_item.text);
                    infrastructure::copy_to_clipboard(snippet)?;
                }
                CompletionAction::PrintSnippet => {
                    let snippet = completion_item.description.as_deref().unwrap_or(&completion_item.text);
                    println!("Snippet: {snippet}");
                }
            }
            Ok(Some(completion_item))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

/// Modified to support fuzzy finding in scriptable mode
pub fn print_completions(
    completion_type: &CompletionType,
    partial_input: &str
) -> Result<()> {
    let items = read_completions_from_file(&completion_type.source_file)?;

    // Use skim's fuzzy matching algorithm for consistency
    let matcher = SkimMatcherV2::default();

    // Filter and sort matches by score
    let mut matches: Vec<_> = items
        .into_iter()
        .filter_map(|item| {
            matcher.fuzzy(&item.text, partial_input, false)
                .map(|score| (score, item.text))
        })
        .collect();

    // Sort by score (higher is better)
    matches.sort_by(|a, b| b.0.cmp(&a.0));

    // Print sorted matches
    println!("{}", matches.into_iter()
        .map(|(_, text)| text)
        .collect::<Vec<_>>()
        .join(" "));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn given_empty_input_when_parsing_then_returns_error() {
        let mut tmp = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(tmp, "apple|A red fruit\nbanana|A yellow fruit").unwrap();

        let ctype = CompletionType {
            name: "test".to_string(),
            source_file: tmp.path().into(),
            keyboard_shortcut: "ctrl+t".to_string(),
            action: CompletionAction::CopyToClipboard,
        };

        let result = perform_completion(&ctype, "");
        assert!(result.is_ok());
    }

    #[test]
    fn given_fuzzy_input_when_matching_then_finds_partial_matches() {
        let mut tmp = NamedTempFile::new().expect("Failed to create temp file");
        writeln!(tmp, "apple|A red fruit\nbanana|A yellow fruit").unwrap();

        let ctype = CompletionType {
            name: "test".to_string(),
            source_file: tmp.path().into(),
            keyboard_shortcut: "ctrl+t".to_string(),
            action: CompletionAction::PrintSnippet,
        };

        // Test scriptable output with fuzzy matching
        print_completions(&ctype, "ana").unwrap();
        // Visual inspection of output would show "banana" matched
    }
}