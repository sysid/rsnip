use anyhow::Result;
use tempfile::NamedTempFile;
use rsnip::application::copy_snippet_to_clipboard;
use rsnip::domain::SnippetType;
use std::io::Write;

#[test]
fn given_existing_snippet_when_copying_then_returns_some() -> Result<()> {
    let mut tmp = NamedTempFile::new()?;
    writeln!(tmp, "--- apple\nA red fruit\n---")?;

    let ctype = SnippetType {
        name: "test".to_string(),
        source_file: tmp.path().into(),
    };

    let result = copy_snippet_to_clipboard(&ctype, "apple")?;
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "apple");
    Ok(())
}

#[test]
fn given_nonexistent_snippet_when_copying_then_returns_none() -> Result<()> {
    let mut tmp = NamedTempFile::new()?;
    writeln!(tmp, "--- apple\nA red fruit\n---")?;

    let ctype = SnippetType {
        name: "test".to_string(),
        source_file: tmp.path().into(),
    };

    let result = copy_snippet_to_clipboard(&ctype, "nonexistent")?;
    assert!(result.is_none());
    Ok(())
}

#[test]
fn given_snippet_without_description_when_copying_then_returns_some() -> Result<()> {
    let mut tmp = NamedTempFile::new()?;
    writeln!(tmp, "--- apple\n---")?;

    let ctype = SnippetType {
        name: "test".to_string(),
        source_file: tmp.path().into(),
    };

    let result = copy_snippet_to_clipboard(&ctype, "apple")?;
    assert!(result.is_some());
    assert_eq!(result.unwrap().name, "apple");
    Ok(())
}
