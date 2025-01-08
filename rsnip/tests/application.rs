use anyhow::Result;
use rsnip::application::{copy_snippet_to_clipboard, find_completion_fuzzy};
use rsnip::config::{get_snippet_type, Settings};
use rsnip::domain::SnippetType;
use std::io::Write;
use tempfile::NamedTempFile;
use rsnip::infrastructure::parse_snippets_file;

#[test]
fn given_empty_input_when_finding_completion_then_returns_none() -> Result<()> {
    let mut tmp = NamedTempFile::new()?;
    writeln!(
        tmp,
        "--- apple\nA red fruit\n---\n--- banana\nA yellow fruit\n---"
    )?;

    let config = Settings::default();
    let ctype = get_snippet_type(&config, "default")?;

    assert!(find_completion_fuzzy(&ctype, "")?.is_none());
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

    let result = copy_snippet_to_clipboard(&ctype, "nonexistent", true)?;
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

    let result = copy_snippet_to_clipboard(&ctype, "apple", true)?;
    assert!(result.is_some());
    let (snippet, _) = result.unwrap();
    assert_eq!(snippet.name, "apple");
    Ok(())
}

#[test]
fn given_template_snippet_when_copying_then_returns_rendered_content() -> Result<()> {
    let mut tmp = NamedTempFile::new()?;
    let template_content = "--- date\n{{current_date|strftime('%Y-%m-%d')}}\n---";  // Removed spaces around {{}}
    println!("Debug - Writing template content: {}", template_content);
    writeln!(tmp, "{}", template_content)?;

    let ctype = SnippetType {
        name: "test".to_string(),
        source_file: tmp.path().into(),
    };

    let snippets = parse_snippets_file(&ctype.source_file)?;
    println!("Debug - Parsed snippets: {:?}", snippets);

    let result = copy_snippet_to_clipboard(&ctype, "date", true)?;
    assert!(result.is_some());
    let (snippet, rendered) = result.unwrap();
    println!("Debug - Snippet: {:?}", snippet);
    println!("Debug - Rendered: {:?}", rendered);

    assert_eq!(snippet.name, "date");

    let is_valid_date = rendered.trim().len() == 10
        && rendered.trim().chars().all(|c| c.is_ascii_digit() || c == '-')
        && rendered.matches('-').count() == 2;

    assert!(is_valid_date, "Rendered date '{}' is not in YYYY-MM-DD format", rendered);
    Ok(())
}

