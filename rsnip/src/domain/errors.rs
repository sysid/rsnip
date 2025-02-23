use thiserror::Error;
use std::path::PathBuf;

#[derive(Error, Debug)]
pub enum SnippetError {
    #[error("Snippet not found: {name}")]
    NotFound { name: String },

    #[error("Invalid snippet format for {} in {}, line {}: {}", name, file.display(), line, reason)]
    InvalidFormat {
        name: String,
        file: PathBuf,
        line: usize,
        reason: String,
    },

    #[error("Failed to read snippet file: {file}")]
    FileError {
        file: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Template error: {0}")]
    TemplateError(#[from] crate::domain::template::errors::TemplateError),

    #[error("Parser error: {0}")]
    ParserError(String),

    #[error("Clipboard error: {0}")]
    ClipboardError(String),
}

// Common result type for snippet operations
pub type SnippetResult<T> = Result<T, SnippetError>;