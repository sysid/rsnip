#[derive(Clone)]
pub struct CompletionItem {
    pub text: String,
    pub description: Option<String>,
}

pub struct CompletionType {
    pub name: String,
    pub source_file: std::path::PathBuf,
    pub keyboard_shortcut: String,
    // TODO: Remove this field in next refactoring
    pub action: CompletionAction,
}

// TODO: Remove this enum in next refactoring
pub enum CompletionAction {
    CopyToClipboard,
    PrintSnippet,
}
