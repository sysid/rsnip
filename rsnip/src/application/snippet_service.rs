use crate::config::Settings;
use crate::domain::parser::SnippetType;
use crate::domain::snippet::Snippet;
use crate::fuzzy;
use crate::infrastructure::clipboard::copy_to_clipboard;
use crate::infrastructure::parsers::SnippetParserFactory;
use crate::template::TemplateEngine;
use anyhow::{Context, Result};
use fuzzy_matcher::skim::SkimMatcherV2;
use std::cmp::Reverse;
use tracing::{debug, instrument};

/// Service for managing snippet operations
#[derive(Debug)]
pub struct SnippetService<'a> {
    template_engine: TemplateEngine,
    config: &'a Settings,
}

impl<'a> SnippetService<'a> {
    /// Creates a new SnippetService
    pub fn new(config: &'a Settings) -> Self {
        Self {
            template_engine: TemplateEngine::new(),
            config,
        }
    }

    /// Get snippets for a given snippet type, handling both concrete and combined types
    #[instrument(level = "debug", skip(self))]
    pub fn get_snippets(&self, snippet_type: &str) -> Result<Vec<Snippet>> {
        // Check if this is a combined type
        if let Some(sources) = self.config.get_combined_sources(snippet_type) {
            debug!("Loading combined snippets from sources: {:?}", sources);
            let mut all_snippets = Vec::new();

            // Load snippets from each source
            for source in sources {
                if let Some(concrete_type) = self.config.get_snippet_type(&source) {
                    let mut source_snippets = self.get_concrete_snippets(&concrete_type)
                        .with_context(|| format!("Failed to load snippets from source '{}'", source))?;
                    all_snippets.append(&mut source_snippets);
                }
            }

            Ok(all_snippets)
        } else {
            // Handle concrete type
            let concrete_type = self.config.get_snippet_type(snippet_type)
                .ok_or_else(|| anyhow::anyhow!("Unknown snippet type: {}", snippet_type))?;
            self.get_concrete_snippets(&concrete_type)
        }
    }

    /// Get snippets from a concrete snippet type
    fn get_concrete_snippets(&self, snippet_type: &SnippetType) -> Result<Vec<Snippet>> {
        debug!(
            "Loading snippets from {}",
            snippet_type.source_file.display()
        );
        let parser = SnippetParserFactory::create(snippet_type.format);
        parser.parse(&snippet_type.source_file).with_context(|| {
            format!(
                "Failed to parse snippets from {}",
                snippet_type.source_file.display()
            )
        })
    }

    /// Finds completions interactively using fuzzy finder
    #[instrument(level = "debug", skip(self))]
    pub fn find_completion_interactive(
        &self,
        completion_type: &str,
        user_input: &str,
    ) -> Result<Option<Snippet>> {
        let items = self.get_snippets(completion_type)?;

        // Early return if no items
        if items.is_empty() {
            return Ok(None);
        }

        let selected_item = fuzzy::run_fuzzy_finder(&items, user_input)?;

        // Return the selected snippet
        Ok(selected_item.and_then(|name| items.iter().find(|item| item.name == name).cloned()))
    }

    /// Find first matching completion non-interactively using fuzzy matching
    #[instrument(level = "debug", skip(self))]
    pub fn find_completion_fuzzy(
        &self,
        completion_type: &str,
        user_input: &str,
    ) -> Result<Option<Snippet>> {
        // Return None for empty input
        if user_input.trim().is_empty() {
            return Ok(None);
        }

        let items = self.get_snippets(completion_type)?;
        let matcher = SkimMatcherV2::default();

        // Find best match using iterator
        Ok(items
            .into_iter()
            .filter_map(|item| {
                matcher
                    .fuzzy(&item.name, user_input, false)
                    .map(|score| (Reverse(score), item))
            })
            .max_by_key(|(score, _)| score.clone())
            .map(|(_, item)| item))
    }

    /// Find a completion using an exact match
    #[instrument(level = "debug", skip(self))]
    pub fn find_completion_exact(
        &self,
        completion_type: &str,
        user_input: &str,
    ) -> Result<Option<Snippet>> {
        if user_input.trim().is_empty() {
            return Ok(None);
        }

        let items = self.get_snippets(completion_type)?;
        Ok(items.into_iter().find(|item| item.name == user_input))
    }

    /// Copy snippet content to clipboard
    #[instrument(level = "debug", skip(self))]
    pub fn copy_snippet_to_clipboard(
        &self,
        completion_type: &str,
        input: &str,
        exact: bool,
    ) -> Result<Option<(Snippet, String)>> {
        let item = if exact {
            self.find_completion_exact(completion_type, input)?
        } else {
            self.find_completion_fuzzy(completion_type, input)?
        };

        if let Some(completion_item) = item.clone() {
            let rendered = self.template_engine.render(&completion_item.content)?;
            copy_to_clipboard(&rendered)?;
            Ok(Some((completion_item, rendered)))
        } else {
            Ok(None)
        }
    }
}
