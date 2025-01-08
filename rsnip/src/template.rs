use crate::domain::{SnippetContent};
use anyhow::Result;
use chrono::{DateTime, Utc};
use minijinja::value::Value;
use minijinja::{Environment, Error, ErrorKind};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, error, instrument, trace};

#[derive(Error, Debug)]
pub enum TemplateError {
    #[error("Template syntax error: {0}")]
    Syntax(String),
    #[error("Template rendering error: {0}")]
    Rendering(String),
    #[error("Context error: {0}")]
    Context(String),
}

// Template context builder for managing template variables and functions
#[derive(Debug, Default)]
pub struct TemplateContext {
    env: Environment<'static>,
}

// TemplateContext implementation
impl TemplateContext {
    pub fn new() -> Self {
        let mut env = Environment::new();

        // Register all template filters
        env.add_filter("strftime", date_format);
        env.add_filter("subtract_days", subtract_days);
        env.add_filter("add_days", add_days);

        Self { env }
    }

    pub fn get_env(&self) -> &Environment<'static> {
        &self.env
    }
}

#[derive(Default, Debug)]
pub struct TemplateEngine {
    context: TemplateContext,
}

impl TemplateEngine {
    pub fn new() -> Self {
        Self {
            context: TemplateContext::new(),
        }
    }

    #[instrument(level = "debug")]
    pub fn render(&self, content: &SnippetContent) -> Result<String, TemplateError> {
        match content {
            SnippetContent::Static(s) => {
                trace!("Static content detected");
                Ok(s.clone())
            },
            SnippetContent::Template { source, compiled } => {
                trace!("Template content detected, source: {}", source);
                // Create context with data
                let mut ctx = HashMap::new();

                // Add current date/time as ISO 8601 string
                let current_date = Utc::now().to_rfc3339();
                trace!("Adding current_date to context: {}", current_date);
                ctx.insert(
                    "current_date".to_string(),
                    Value::from(current_date),
                );

                // Add environment variables with stable keys
                for (key, value) in std::env::vars() {
                    let env_key = format!("env_{}", key);
                    ctx.insert(env_key, Value::from(value));
                }

                // Use cached template or compile new one
                let template = if let Some(t) = compiled {
                    trace!("Using cached template");
                    t
                } else {
                    trace!("Compiling new template");
                    &self
                        .context
                        .get_env()
                        .template_from_str(source)
                        .map_err(|e| {
                            TemplateError::Syntax(e.to_string())
                        })?
                };

                let result = template
                    .render(&ctx)
                    .map_err(|e| {
                        TemplateError::Rendering(e.to_string())
                    });
                debug!("Render result: {:?}", result);
                result
            }
        }
    }
}

// Filter implementations
fn date_format(value: Value, args: &[Value]) -> Result<Value, Error> {
    let date_str = value
        .as_str()
        .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "Expected string date"))?;
    let format = args.get(0).and_then(|v| v.as_str()).unwrap_or("%Y-%m-%d");

    // Parse date and format
    let date = DateTime::parse_from_rfc3339(date_str)
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("Invalid date: {}", e)))?
        .with_timezone(&Utc);

    Ok(Value::from(date.format(format).to_string()))
}

fn subtract_days(value: Value, args: &[Value]) -> Result<Value, Error> {
    // Always expect string input
    let date_str = value
        .as_str()
        .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "Expected date string"))?;

    let date = DateTime::parse_from_rfc3339(date_str)
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("Invalid date: {}", e)))?;

    let days = args.get(0).and_then(|v| v.as_i64()).unwrap_or(0);

    let new_date = date - chrono::Duration::days(days);
    Ok(Value::from(new_date.to_rfc3339()))
}

fn add_days(value: Value, args: &[Value]) -> Result<Value, Error> {
    // Always expect string input
    let date_str = value
        .as_str()
        .ok_or_else(|| Error::new(ErrorKind::InvalidOperation, "Expected date string"))?;

    let date = DateTime::parse_from_rfc3339(date_str)
        .map_err(|e| Error::new(ErrorKind::InvalidOperation, format!("Invalid date: {}", e)))?;

    let days = args.get(0).and_then(|v| v.as_i64()).unwrap_or(0);

    let new_date = date + chrono::Duration::days(days);
    Ok(Value::from(new_date.to_rfc3339()))
}