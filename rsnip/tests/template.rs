use chrono::Utc;
use rsnip::domain::SnippetContent;
use rsnip::template::TemplateEngine;

#[test]
fn test_static_content() {
    let engine = TemplateEngine::new();
    let content = SnippetContent::Static("Hello World".to_string());
    assert_eq!(engine.render(&content).unwrap(), "Hello World");
}

#[test]
fn test_template_date() {
    let engine = TemplateEngine::new(); // Changed from default()
    let content = SnippetContent::Template {
        source: "{{ current_date | strftime('%Y-%m-%d') }}".to_string(),
        compiled: None,
    };
    let result = engine.render(&content).unwrap();
    // Verify it matches YYYY-MM-DD format
    assert!(result.chars().count() == 10);
    assert!(result.contains('-'));
}

#[test]
fn test_template_date_arithmetic() {
    let engine = TemplateEngine::new(); // Changed from default()
    let content = SnippetContent::Template {
        source: "{{ current_date | subtract_days(7) | strftime('%Y-%m-%d') }}".to_string(),
        compiled: None,
    };
    let result = engine.render(&content).unwrap();
    // Verify it matches YYYY-MM-DD format and is a week ago
    let today = Utc::now();
    let week_ago = today - chrono::Duration::days(7);
    assert_eq!(result, week_ago.format("%Y-%m-%d").to_string());
}

#[test]
fn test_template_env() {
    std::env::set_var("TEST_VAR", "test_value");
    let engine = TemplateEngine::new();
    let content = SnippetContent::Template {
        source: "{{ env_TEST_VAR }}".to_string(),
        compiled: None,
    };
    assert_eq!(engine.render(&content).unwrap(), "test_value");
}