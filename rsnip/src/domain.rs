pub struct CompletionItem {
    pub text: String,
    pub description: Option<String>,
}

pub struct CompletionType {
    pub name: String,
    pub source_file: std::path::PathBuf,
    pub keyboard_shortcut: String,
    pub action: CompletionAction,
}

pub enum CompletionAction {
    CopyToClipboard,
    PrintSnippet,
    // Add more actions if needed
}
