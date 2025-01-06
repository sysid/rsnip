#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct SnippetType {
    pub name: String,
    pub source_file: std::path::PathBuf,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Snippet {
    pub name: String,
    pub snippet: Option<String>,
}

