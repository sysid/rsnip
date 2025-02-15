use chrono::Utc;
use rsnip::domain::content::SnippetContent;
use rsnip::domain::template::interface::TemplateEngine;
use rsnip::infrastructure::minijinja::{MiniJinjaEngine, SafeShellExecutor};

// Helper function to create a template engine instance
fn create_engine() -> impl TemplateEngine {
    MiniJinjaEngine::new(Box::new(SafeShellExecutor::new()))
}

#[test]
fn given_static_content_when_rendering_then_returns_unchanged() {
    // Arrange
    let engine = create_engine();
    let content = SnippetContent::Static("Hello World".to_string());

    // Act
    let result = engine.render(&content).unwrap();

    // Assert
    assert_eq!(result, "Hello World");
}

#[test]
fn given_template_with_date_when_rendering_then_formats_date() {
    // Arrange
    let engine = create_engine();
    let content = SnippetContent::Template {
        source: "{{ current_date | strftime('%Y-%m-%d') }}".to_string(),
        compiled: None,
    };

    // Act
    let result = engine.render(&content).unwrap();

    // Assert
    let date_regex = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap();
    assert!(date_regex.is_match(&result), "Date format should be YYYY-MM-DD");
}

#[test]
fn given_template_with_date_arithmetic_when_rendering_then_calculates_correctly() {
    // Arrange
    let engine = create_engine();
    let content = SnippetContent::Template {
        source: "{{ current_date | subtract_days(7) | strftime('%Y-%m-%d') }}".to_string(),
        compiled: None,
    };

    // Act
    let result = engine.render(&content).unwrap();

    // Assert
    let today = Utc::now();
    let week_ago = today - chrono::Duration::days(7);
    assert_eq!(result, week_ago.format("%Y-%m-%d").to_string());
}

#[test]
fn given_template_with_env_var_when_rendering_then_substitutes_value() {
    // Arrange
    std::env::set_var("TEST_VAR", "test_value");
    let engine = create_engine();
    let content = SnippetContent::Template {
        source: "{{ env_TEST_VAR }}".to_string(),
        compiled: None,
    };

    // Act
    let result = engine.render(&content).unwrap();

    // Assert
    assert_eq!(result, "test_value");
}

#[test]
fn given_template_with_shell_command_when_rendering_then_executes() {
    // Arrange
    let engine = create_engine();
    let content = SnippetContent::Template {
        source: "{{ 'echo Hello' | shell }}".to_string(),
        compiled: None,
    };

    // Act
    let result = engine.render(&content).unwrap();

    // Assert
    assert_eq!(result, "Hello");
}

#[test]
fn given_template_with_invalid_shell_command_when_rendering_then_returns_error() {
    // Arrange
    let engine = create_engine();
    let content = SnippetContent::Template {
        source: "{{ 'nonexistent_command' | shell }}".to_string(),
        compiled: None,
    };

    // Act & Assert
    assert!(engine.render(&content).is_err());
}

#[test]
fn given_template_with_dangerous_shell_commands_then_returns_error() {
    // Arrange
    let engine = create_engine();
    let dangerous_commands = vec![
        "echo hello; rm -rf /",
        "echo hello | rm -rf /",
        "echo hello && rm -rf /",
        "sudo echo hello",
        "`echo hello`",
        "$(echo hello)",
    ];

    // Act & Assert
    for cmd in dangerous_commands {
        let content = SnippetContent::Template {
            source: format!("{{{{ '{}' | shell }}}}", cmd),
            compiled: None,
        };
        assert!(
            engine.render(&content).is_err(),
            "Should reject dangerous command: {}",
            cmd
        );
    }
}

#[test]
fn given_template_with_add_days_when_rendering_then_calculates_correctly() {
    // Arrange
    let engine = create_engine();
    let content = SnippetContent::Template {
        source: "{{ current_date | add_days(7) | strftime('%Y-%m-%d') }}".to_string(),
        compiled: None,
    };

    // Act
    let result = engine.render(&content).unwrap();

    // Assert
    let today = Utc::now();
    let next_week = today + chrono::Duration::days(7);
    assert_eq!(result, next_week.format("%Y-%m-%d").to_string());
}