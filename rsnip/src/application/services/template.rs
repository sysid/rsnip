// application/services/template.rs
use crate::domain::snippet::Snippet;
use crate::domain::template::interface::TemplateEngine;
use crate::infrastructure::clipboard::copy_to_clipboard;
use anyhow::Result;
use tracing::instrument;

pub struct TemplateProcessingService {
    template_engine: Box<dyn TemplateEngine>,
}

impl TemplateProcessingService {
    pub fn new(template_engine: Box<dyn TemplateEngine>) -> Self {
        Self { template_engine }
    }

    #[instrument(level = "debug", skip(self))]
    pub fn process_and_copy(&self, snippet: &Snippet) -> Result<String> {
        let rendered = self.template_engine.render(&snippet.content)?;
        copy_to_clipboard(&rendered)?;
        Ok(rendered)
    }
}