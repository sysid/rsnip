use crate::domain::content::SnippetContent;

#[derive(Clone, Debug, PartialEq)]
pub struct Snippet {
    pub name: String,
    pub content: SnippetContent,
    pub comments: Vec<String>,
}

