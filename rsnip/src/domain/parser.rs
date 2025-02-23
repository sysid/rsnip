use crate::domain::snippet::Snippet;
use crate::domain::errors::SnippetResult;
use std::path::Path;
use std::path::PathBuf;

/// Trait defining the interface for snippet parsers
pub trait SnippetParser: Send + Sync {
    /// Parse snippets from a file at the given path
    fn parse(&self, path: &Path) -> SnippetResult<Vec<Snippet>>;
}

/// Format of the snippet file
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SnippetFormat {
    Default, // The original rsnip format
    Scls,    // simple-completion-language-server format
    VCode,   // Visual Studio Code format
}

impl SnippetFormat {
    pub fn from_str(format: &str) -> Option<Self> {
        match format.to_lowercase().as_str() {
            "default" => Some(Self::Default),
            "scls" => Some(Self::Scls),
            "vcode" => Some(Self::VCode),
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
