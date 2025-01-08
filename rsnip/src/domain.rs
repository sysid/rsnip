use minijinja::Template;
use std::path::PathBuf;
use tracing::{instrument, trace};

#[derive(Clone, Debug, PartialEq)]
pub struct SnippetType {
    pub name: String,
    pub source_file: PathBuf,
}

#[derive(Clone, Debug)]
pub enum SnippetContent {
    Static(String),
    Template {
        source: String,
        // We don't clone the compiled template, just store it for performance
        compiled: Option<Template<'static, 'static>>,
    }
}

// Manual implementation of PartialEq that ignores the compiled template
impl PartialEq for SnippetContent {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Static(a), Self::Static(b)) => a == b,
            (Self::Template { source: a, .. }, Self::Template { source: b, .. }) => a == b,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Snippet {
    pub name: String,
    pub content: SnippetContent,
}

// Implement template detection
impl SnippetContent {
    #[instrument(level = "debug")]
    pub fn new(content: String) -> Self {
        trace!("SnippetContent::new called with: {}", content);
        if content.contains("{{") && content.contains("}}") {
            trace!("Detected as template");
            SnippetContent::Template {
                source: content,
                compiled: None,
            }
        } else {
            trace!("Detected as static");
            SnippetContent::Static(content)
        }
    }

    pub fn get_content(&self) -> &str {
        match self {
            SnippetContent::Static(s) => s,
            SnippetContent::Template { source, .. } => source,
        }
    }
}