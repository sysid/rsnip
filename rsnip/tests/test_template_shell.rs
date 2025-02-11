use rsnip::domain::content::SnippetContent;
use rsnip::template::TemplateEngine;

#[test]
fn test_shell_command_echo() {
    let engine = TemplateEngine::new();
    let content = SnippetContent::Template {
        source: "{{ 'echo Hello' | shell }}".to_string(),
        compiled: None,
    };
    assert_eq!(engine.render(&content).unwrap(), "Hello");
}

#[test]
fn test_shell_command_date() {
    let engine = TemplateEngine::new();
    let content = SnippetContent::Template {
        source: "{{ 'date +%Y' | shell }}".to_string(),
        compiled: None,
    };
    let result = engine.render(&content).unwrap();
    // Should match current year
    assert_eq!(result, chrono::Utc::now().format("%Y").to_string());
}

#[test]
fn test_shell_command_invalid() {
    let engine = TemplateEngine::new();
    let content = SnippetContent::Template {
        source: "{{ 'nonexistent_command' | shell }}".to_string(),
        compiled: None,
    };
    println!("{:?}", content);
    let result = engine.render(&content);
    println!("{:?}", result);
    assert!(engine.render(&content).is_err());
}

#[test]
fn test_shell_command_security() {
    let engine = TemplateEngine::new();
    // Test command injection attempts
    let dangerous_commands = vec![
        "echo hello; xx -rf /",
        "echo hello | xx -rf /",
        "echo hello && xx -rf /",
    ];

    for cmd in dangerous_commands {
        let content = SnippetContent::Template {
            source: format!("{{{{ '{}' | shell }}}}", cmd),
            compiled: None,
        };
        println!("{:?}", content);
        assert!(
            engine.render(&content).is_err(),
            "Should reject dangerous command: {}",
            cmd
        );
    }
}
