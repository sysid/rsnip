// domain/template/model.rs
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// todo: delete?

#[derive(Clone, Debug)]
pub struct TemplateContext {
    pub(crate) variables: HashMap<String, TemplateValue>,
}

#[derive(Clone, Debug)]
pub enum TemplateValue {
    String(String),
    DateTime(DateTime<Utc>),
    Number(f64),
    Bool(bool),
}

impl TemplateContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn insert<K: Into<String>, V: Into<TemplateValue>>(&mut self, key: K, value: V) {
        self.variables.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&TemplateValue> {
        self.variables.get(key)
    }
}

// Value conversions
impl From<String> for TemplateValue {
    fn from(s: String) -> Self {
        TemplateValue::String(s)
    }
}

impl From<DateTime<Utc>> for TemplateValue {
    fn from(dt: DateTime<Utc>) -> Self {
        TemplateValue::DateTime(dt)
    }
}
