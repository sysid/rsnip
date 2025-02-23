// application/services/completion.rs
use crate::domain::snippet::Snippet;
use crate::infrastructure::fuzzy;
use anyhow::Result;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::cmp::Reverse;
use tracing::instrument;

#[derive(Debug)]
pub struct CompletionService;

impl CompletionService {
    pub fn new() -> Self {
        Self
    }

    #[instrument(level = "debug")]
    pub fn find_completion_interactive(&self, items: &[Snippet], user_input: &str) -> Result<Option<Snippet>> {
        if items.is_empty() {
            return Ok(None);
        }

        let selected_item = fuzzy::run_fuzzy_finder(items, user_input)?;
        Ok(selected_item.and_then(|name| items.iter().find(|item| item.name == name).cloned()))
    }

    #[instrument(level = "debug")]
    pub fn find_completion_fuzzy(&self, items: &[Snippet], user_input: &str) -> Option<Snippet> {
        if user_input.trim().is_empty() {
            return None;
        }

        let matcher = SkimMatcherV2::default();
        items.iter()
            .filter_map(|item| {
                matcher.fuzzy(&item.name, user_input, false)
                    .map(|score| (Reverse(score), item))
            })
            .max_by_key(|(score, _)| score.clone())
            .map(|(_, item)| item.clone())
    }

    #[instrument(level = "debug")]
    pub fn find_completion_exact(&self, items: &[Snippet], user_input: &str) -> Option<Snippet> {
        if user_input.trim().is_empty() {
            return None;
        }

        items.iter()
            .find(|item| item.name == user_input)
            .cloned()
    }
}