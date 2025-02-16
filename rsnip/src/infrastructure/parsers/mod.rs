mod default;
mod scls;
mod vcode;

pub use default::DefaultSnippetParser;
pub use scls::SclsSnippetParser;
pub use vcode::VCodeSnippetParser;

use crate::domain::parser::{SnippetFormat, SnippetParser};
use std::sync::Arc;
use tracing::instrument;

/// Factory for creating appropriate parser instances
pub struct SnippetParserFactory;

impl SnippetParserFactory {
    #[instrument(level = "debug")]
    pub fn create(format: SnippetFormat) -> Arc<dyn SnippetParser> {
        match format {
            SnippetFormat::Default => Arc::new(DefaultSnippetParser::new()),
            SnippetFormat::Scls => Arc::new(SclsSnippetParser::new()),
            SnippetFormat::VCode => Arc::new(VCodeSnippetParser::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn given_format_when_creating_parser_then_returns_correct_implementation() {
        let default_parser = SnippetParserFactory::create(SnippetFormat::Default);
        let scls_parser = SnippetParserFactory::create(SnippetFormat::Scls);

        // Type checking is enough here as the concrete types are private
        assert!(default_parser.parse(&PathBuf::from("dummy")).is_err());
        assert!(scls_parser.parse(&PathBuf::from("dummy")).is_err());
    }

    #[test]
    fn given_default_format_file_when_parsing_then_succeeds() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(file, "--- test\ncontent\n---")?;

        let parser = SnippetParserFactory::create(SnippetFormat::Default);
        let snippets = parser.parse(file.path())?;

        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].name, "test");
        Ok(())
    }

    #[test]
    fn given_scls_format_file_when_parsing_then_succeeds() -> Result<()> {
        let mut file = NamedTempFile::new()?;
        writeln!(
            file,
            r#"
            [[snippets]]
            prefix = "test"
            body = "content"
        "#
        )?;

        let parser = SnippetParserFactory::create(SnippetFormat::Scls);
        let snippets = parser.parse(file.path())?;

        assert_eq!(snippets.len(), 1);
        assert_eq!(snippets[0].name, "test");
        Ok(())
    }
}
