// Updated snippet_service.rs as facade
use crate::application::services::{SnippetManagementService, CompletionService, TemplateProcessingService};
use crate::config::Settings;
use crate::domain::snippet::Snippet;
use anyhow::Result;
use tracing::instrument;
use crate::domain::template::interface::TemplateEngine;

pub struct SnippetService<'a> {
    management: SnippetManagementService<'a>,
    completion: CompletionService,
    template: TemplateProcessingService,
}

impl<'a> SnippetService<'a> {
    pub fn new(template_engine: Box<dyn TemplateEngine>, config: &'a Settings) -> Self {
        Self {
            management: SnippetManagementService::new(config),
            completion: CompletionService::new(),
            template: TemplateProcessingService::new(template_engine),
        }
    }

    pub fn get_snippets(&self, snippet_type: &str) -> Result<Vec<Snippet>> {
        self.management.get_snippets(snippet_type)
    }

    #[instrument(level = "debug", skip(self))]
    pub fn find_completion_interactive(&self, completion_type: &str, user_input: &str) -> Result<Option<Snippet>> {
        let items = self.get_snippets(completion_type)?;
        self.completion.find_completion_interactive(&items, user_input)
    }

    #[instrument(level = "debug", skip(self))]
    pub fn find_completion_fuzzy(&self, completion_type: &str, user_input: &str) -> Result<Option<Snippet>> {
        let items = self.get_snippets(completion_type)?;
        Ok(self.completion.find_completion_fuzzy(&items, user_input))
    }

    #[instrument(level = "debug", skip(self))]
    pub fn find_completion_exact(&self, completion_type: &str, user_input: &str) -> Result<Option<Snippet>> {
        let items = self.get_snippets(completion_type)?;
        Ok(self.completion.find_completion_exact(&items, user_input))
    }

    #[instrument(level = "debug", skip(self))]
    pub fn copy_snippet_to_clipboard(&self, completion_type: &str, input: &str, exact: bool) -> Result<Option<(Snippet, String)>> {
        let item = if exact {
            self.find_completion_exact(completion_type, input)?
        } else {
            self.find_completion_fuzzy(completion_type, input)?
        };

        if let Some(completion_item) = item.clone() {
            let rendered = self.template.process_and_copy(&completion_item)?;
            Ok(Some((completion_item, rendered)))
        } else {
            Ok(None)
        }
    }
}