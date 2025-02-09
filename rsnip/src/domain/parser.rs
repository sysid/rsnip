use std::path::Path;
use anyhow::Result;
use std::path::PathBuf;
use crate::domain::tochange::Snippet;

/// Trait defining the interface for snippet parsers
pub trait SnippetParser: Send + Sync {
    /// Parse snippets from a file at the given path
    fn parse(&self, path: &Path) -> Result<Vec<Snippet>>;
}

/// Format of the snippet file
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnippetFormat {
    Default,  // The original rsnip format
    Scls,     // simple-completion-language-server format
}

impl SnippetFormat {
    pub fn from_str(format: &str) -> Option<Self> {
        match format.to_lowercase().as_str() {
            "default" => Some(Self::Default),
            "scls" => Some(Self::Scls),
            _ => None,
        }
    }
}

/// Type representing a collection of snippets
#[derive(Clone, Debug, PartialEq)]
pub struct SnippetType {
    pub name: String,
    pub source_file: PathBuf,
    pub format: SnippetFormat,
}