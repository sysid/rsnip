use crate::domain::parser::SnippetType;
use crate::domain::snippet::Snippet;
use crate::fuzzy;
use crate::infrastructure;
use crate::infrastructure::clipboard::copy_to_clipboard;
use crate::infrastructure::parsers::SnippetParserFactory;
use crate::template::TemplateEngine;
use anyhow::{Context, Result};
use fuzzy_matcher::skim::SkimMatcherV2;
use std::cmp::Reverse;
use tracing::{debug, instrument};

/// Service for managing snippet operations
#[derive(Debug, Default)]
pub struct SnippetService {
    template_engine: TemplateEngine,
}

impl SnippetService {
    /// Creates a new SnippetService
    pub fn new() -> Self {
        Self {
            template_engine: TemplateEngine::new(),
        }
    }

    /// Get snippets for a given snippet type
    #[instrument(level = "debug", skip(self))]
    pub fn get_snippets(&self, snippet_type: &SnippetType) -> Result<Vec<Snippet>> {
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
        completion_type: &SnippetType,
        user_input: &str,
    ) -> Result<Option<Snippet>> {
        let items = self.get_snippets(completion_type)?;

        // Early return if no items
        if items.is_empty() {
            return Ok(None);
        }

        let selected_item = fuzzy::run_fuzzy_finder(&items, completion_type, user_input)?;

        // Return the selected snippet
        Ok(selected_item.and_then(|name| items.iter().find(|item| item.name == name).cloned()))
    }

    /// Find first matching completion non-interactively using fuzzy matching
    #[instrument(level = "debug", skip(self))]
    pub fn find_completion_fuzzy(
        &self,
        completion_type: &SnippetType,
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
        completion_type: &SnippetType,
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
        completion_type: &SnippetType,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::parser::SnippetFormat;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_snippet_type(path: &std::path::Path) -> SnippetType {
        SnippetType {
            name: "test".to_string(),
            source_file: path.to_path_buf(),
            format: SnippetFormat::Default,
        }
    }

    #[test]
    fn given_valid_snippet_file_when_getting_snippets_then_returns_snippets() -> Result<()> {
        // Arrange
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "--- test\nContent\n---")?;
        let snippet_type = create_test_snippet_type(temp_file.path());
        let service = SnippetService::new();

        // Act
        let snippets = service.get_snippets(&snippet_type)?;

        // Assert
        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].name, "test");
        assert_eq!(snippets[0].content.get_content(), "Content");

        Ok(())
    }

    #[test]
    fn given_exact_match_when_finding_completion_then_returns_snippet() -> Result<()> {
        // Arrange
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "--- test\nContent\n---")?;
        let snippet_type = create_test_snippet_type(temp_file.path());
        let service = SnippetService::new();

        // Act
        let result = service.find_completion_exact(&snippet_type, "test")?;

        // Assert
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test");
        Ok(())
    }

    #[test]
    fn given_fuzzy_match_when_finding_completion_then_returns_best_match() -> Result<()> {
        // Arrange
        let mut temp_file = NamedTempFile::new()?;
        writeln!(
            temp_file,
            "--- test\nContent\n---\n--- testing\nContent2\n---"
        )?;
        let snippet_type = create_test_snippet_type(temp_file.path());
        let service = SnippetService::new();

        // Act
        let result = service.find_completion_fuzzy(&snippet_type, "tst")?;

        // Assert
        assert!(result.is_some());
        assert!(result.unwrap().name.contains("test"));
        Ok(())
    }

    #[test]
    fn given_empty_input_when_finding_completion_then_returns_none() -> Result<()> {
        // Arrange
        let temp_file = NamedTempFile::new()?;
        let snippet_type = create_test_snippet_type(temp_file.path());
        let service = SnippetService::new();

        // Act & Assert
        assert!(service
            .find_completion_exact(&snippet_type, "")
            .unwrap()
            .is_none());
        assert!(service
            .find_completion_fuzzy(&snippet_type, "")
            .unwrap()
            .is_none());
        Ok(())
    }
}
