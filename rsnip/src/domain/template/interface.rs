// domain/template/interface.rs
use crate::domain::content::SnippetContent;
use crate::domain::template::errors::TemplateError;
use anyhow::Result;

pub trait TemplateEngine: Send + Sync {
    fn render(&self, content: &SnippetContent) -> Result<String, TemplateError>;
}
// in domain/template/interface.rs
pub trait ShellCommandExecutor: Send + Sync {
    fn execute(&self, command: &str) -> Result<String, TemplateError>;
    fn box_clone(&self) -> Box<dyn ShellCommandExecutor>;
}
