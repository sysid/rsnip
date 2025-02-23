// application/services/management.rs
use crate::config::Settings;
use crate::domain::parser::SnippetType;
use crate::domain::snippet::Snippet;
use crate::infrastructure::parsers::SnippetParserFactory;
use anyhow::{Context, Result};
use tracing::{debug, instrument};

pub struct SnippetManagementService<'a> {
    config: &'a Settings,
}

impl<'a> SnippetManagementService<'a> {
    pub fn new(config: &'a Settings) -> Self {
        Self { config }
    }

    /// Get snippets for a given snippet type, handling both concrete and combined types
    #[instrument(level = "debug", skip(self))]
    pub fn get_snippets(&self, snippet_type: &str) -> Result<Vec<Snippet>> {
        if let Some(sources) = self.config.get_combined_sources(snippet_type) {
            debug!("Loading combined snippets from sources: {:?}", sources);
            let mut all_snippets = Vec::new();

            for source in sources {
                if let Some(concrete_type) = self.config.get_snippet_type(&source) {
                    let mut source_snippets = self.get_concrete_snippets(&concrete_type)
                        .with_context(|| format!("Failed to load snippets from source '{}'", source))?;
                    all_snippets.append(&mut source_snippets);
                }
            }

            Ok(all_snippets)
        } else {
            let concrete_type = self.config.get_snippet_type(snippet_type)
                .ok_or_else(|| anyhow::anyhow!("Unknown snippet type: {}", snippet_type))?;
            self.get_concrete_snippets(&concrete_type)
        }
    }

    fn get_concrete_snippets(&self, snippet_type: &SnippetType) -> Result<Vec<Snippet>> {
        debug!("Loading snippets from {}", snippet_type.source_file.display());
        let parser = SnippetParserFactory::create(snippet_type.format);
        parser.parse(&snippet_type.source_file)
            .with_context(|| format!("Failed to parse snippets from {}", snippet_type.source_file.display()))
    }
}