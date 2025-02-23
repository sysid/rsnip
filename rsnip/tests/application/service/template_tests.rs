// application/services/tests/template_tests.rs
use anyhow::Result;
use rsnip::application::services::TemplateProcessingService;
use rsnip::domain::content::SnippetContent;
use rsnip::domain::snippet::Snippet;
use rsnip::domain::template::errors::TemplateError;
use rsnip::domain::template::interface::TemplateEngine;

struct MockTemplateEngine;

impl TemplateEngine for MockTemplateEngine {
    fn render(&self, content: &SnippetContent) -> Result<String, TemplateError> {
        match content {
            SnippetContent::Static(s) => Ok(s.clone()),
            SnippetContent::Template { source, .. } => Ok(format!("Rendered: {}", source)),
        }
    }
}

#[test]
fn given_static_content_when_processing_then_copies_unchanged() -> Result<()> {
    // Arrange
    let service = TemplateProcessingService::new(Box::new(MockTemplateEngine));
    let snippet = Snippet {
        name: "test".to_string(),
        content: SnippetContent::Static("static content".to_string()),
        comments: vec![],
    };

    // Act
    let result = service.process_and_copy(&snippet)?;

    // Assert
    assert_eq!(result, "static content");
    Ok(())
}