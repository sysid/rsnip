// infrastructure/minijinja.rs
use crate::domain::{
    content::SnippetContent,
    template::{
        errors::TemplateError,
        interface::{ShellCommandExecutor, TemplateEngine},
    },
};
use chrono::{DateTime, Utc};
use minijinja::{Environment, Error, ErrorKind, Value};
use std::collections::HashMap;
use std::process::Command;
use tracing::{info};

pub struct MiniJinjaEngine {
    env: Environment<'static>,
}

impl MiniJinjaEngine {
    pub fn new(shell_executor: Box<dyn ShellCommandExecutor>) -> Self {
        let mut env = Environment::new();

        // Register template filters
        env.add_filter("strftime", date_format);
        env.add_filter("subtract_days", subtract_days);
        env.add_filter("add_days", add_days);

        // Create shell filter with captured executor
        let shell_executor_clone = shell_executor.box_clone();
        env.add_filter("shell", move |value: Value| {
            let cmd = value.as_str().ok_or_else(|| {
                Error::new(ErrorKind::InvalidOperation, "Expected string command")
            })?;

            match shell_executor_clone.execute(cmd) {
                Ok(result) => Ok(Value::from(result)),
                Err(e) => Err(Error::new(ErrorKind::InvalidOperation, e.to_string())),
            }
        });

        Self {
            env,
        }
    }

    fn create_context(&self) -> HashMap<String, Value> {
        let mut context = HashMap::new();

        // Add current date/time
        context.insert(
            "current_date".to_string(),
            Value::from(Utc::now().to_rfc3339()),
        );

        // Add environment variables
        for (key, value) in std::env::vars() {
            context.insert(format!("env_{}", key), Value::from(value));
        }

        context
    }
}

impl TemplateEngine for MiniJinjaEngine {
    fn render(&self, content: &SnippetContent) -> Result<String, TemplateError> {
        match content {
            SnippetContent::Static(s) => Ok(s.clone()),
            SnippetContent::Template { source, .. } => {
                let template = self
                    .env
                    .template_from_str(source)
                    .map_err(|e| TemplateError::Syntax(e.to_string()))?;

                let context = self.create_context();

                template
                    .render(context)
                    .map_err(|e| TemplateError::Rendering(e.to_string()))
            }
        }
    }
}

// Safe shell executor implementation
#[derive(Clone, Debug)]
pub struct SafeShellExecutor;

impl SafeShellExecutor {
    pub fn new() -> Self {
        Self
    }

    fn is_command_safe(&self, cmd: &str) -> bool {
        let dangerous_patterns = [
            ";", "|", "&", ">", "<", "`", "$", "(", ")", "{", "}", "[", "]", "sudo", "rm", "mv",
            "cp", "dd", "mkfs", "fork", "kill",
        ];

        !dangerous_patterns
            .iter()
            .any(|pattern| cmd.contains(pattern))
    }
}

impl ShellCommandExecutor for SafeShellExecutor {
    fn execute(&self, cmd: &str) -> Result<String, TemplateError> {
        info!("Executing shell command: {}", cmd);

        if !self.is_command_safe(cmd) {
            return Err(TemplateError::Shell(
                "Command contains forbidden patterns".to_string(),
            ));
        }

        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| TemplateError::Shell(format!("Failed to execute command: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(TemplateError::Shell(format!("Command failed: {}", error)));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }
    fn box_clone(&self) -> Box<dyn ShellCommandExecutor> {
        Box::new(self.clone())
    }
}

// Filter implementations
fn date_format(value: Value, args: &[Value]) -> Result<Value, Error> {
    let date_str = value
        .as_str()
        .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "Expected string date"))?;
    let format = args.first().and_then(|v| v.as_str()).unwrap_or("%Y-%m-%d");

    let date = DateTime::parse_from_rfc3339(date_str)
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("Invalid date: {}", e)))?
        .with_timezone(&Utc);

    Ok(Value::from(date.format(format).to_string()))
}

fn subtract_days(value: Value, args: &[Value]) -> Result<Value, Error> {
    let date_str = value
        .as_str()
        .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "Expected date string"))?;

    let date = DateTime::parse_from_rfc3339(date_str)
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("Invalid date: {}", e)))?;

    let days = args.first().and_then(|v| v.as_i64()).unwrap_or(0);
    let new_date = date - chrono::Duration::days(days);

    Ok(Value::from(new_date.to_rfc3339()))
}

fn add_days(value: Value, args: &[Value]) -> Result<Value, Error> {
    let date_str = value
        .as_str()
        .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "Expected date string"))?;

    let date = DateTime::parse_from_rfc3339(date_str)
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("Invalid date: {}", e)))?;

    let days = args.first().and_then(|v| v.as_i64()).unwrap_or(0);
    let new_date = date + chrono::Duration::days(days);

    Ok(Value::from(new_date.to_rfc3339()))
}

