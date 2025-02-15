use crate::domain::parser::{SnippetFormat, SnippetType};
use crate::util::path_utils::expand_path;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, instrument, trace};

#[derive(Debug, Deserialize)]
pub struct Settings {
    #[serde(default)]
    pub snippet_types: HashMap<String, SnippetTypeConfig>,
    #[serde(default = "default_config_paths")]
    pub config_paths: Vec<PathBuf>,
    // Track which config file is active
    #[serde(skip)]
    pub active_config_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SnippetTypeConfig {
    Concrete {
        source_file: PathBuf,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        alias: Option<String>,
        #[serde(default = "default_format")]
        format: String,
    },
    Combined {
        sources: Vec<String>,
        #[serde(default)]
        description: Option<String>,
        #[serde(default)]
        alias: Option<String>,
    },
}

fn default_format() -> String {
    "default".to_string()
}

fn default_config_paths() -> Vec<PathBuf> {
    vec![
        dirs::config_dir().map(|p| p.join("rsnip/config.toml")),
        dirs::home_dir().map(|p| p.join(".config/rsnip/config.toml")),
        Some(PathBuf::from("/etc/rsnip/config.toml")),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn default_snippet_types() -> HashMap<String, SnippetTypeConfig> {
    let mut types = HashMap::new();
    types.insert(
        "default".to_string(),
        SnippetTypeConfig::Concrete {
            source_file: PathBuf::from("completion_source.txt"),
            description: Some("Default snippet type".to_string()),
            alias: None,
            format: "default".to_string(),
        },
    );
    types
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            snippet_types: default_snippet_types(),
            config_paths: default_config_paths(),
            active_config_path: None,
        }
    }
}

impl Settings {
    #[instrument(level = "trace")]
    pub fn new() -> Result<Self> {
        let mut builder = config::Config::builder();
        let mut active_path = None;

        // Try loading from config paths first
        for path in default_config_paths() {
            trace!("Trying to load config from: {:?}", path);
            if path.exists() {
                builder = builder.add_source(config::File::from(path.as_path()));
                active_path = Some(path.clone());
                debug!("Loaded config from: {:?}", path);
                break; // Use first found config file
            }
        }

        // Only load default config if no custom config was found
        if active_path.is_none() {
            debug!("No custom config found, using default config");
            builder = builder.add_source(config::File::from_str(
                include_str!("default_config.toml"),
                config::FileFormat::Toml,
            ));
        }

        // Override with environment variables
        builder = builder.add_source(
            config::Environment::with_prefix("RSNIP")
                .separator("_")
                .try_parsing(true),
        );

        let config = builder.build()?;
        let mut settings: Settings = config.try_deserialize()?;

        // Store the active config path
        settings.active_config_path = active_path;

        // Expand paths in concrete snippet types
        for config in settings.snippet_types.values_mut() {
            if let SnippetTypeConfig::Concrete { source_file, .. } = config {
                *source_file = expand_path(&source_file)?;
            }
        }

        debug!("Loaded config: {:?}", settings);
        Ok(settings)
    }

    pub fn get_snippet_type(&self, name: &str) -> Option<SnippetType> {
        match self.snippet_types.get(name) {
            Some(SnippetTypeConfig::Concrete { source_file, format, .. }) => {
                let format = SnippetFormat::from_str(format)
                    .unwrap_or(SnippetFormat::Default);

                Some(SnippetType {
                    name: name.to_string(),
                    source_file: source_file.clone(),
                    format,
                })
            }
            Some(SnippetTypeConfig::Combined { .. }) => {
                // Return None for combined types as they are handled differently
                None
            }
            None => None,
        }
    }

    pub fn get_combined_sources(&self, name: &str) -> Option<Vec<String>> {
        match self.snippet_types.get(name) {
            Some(SnippetTypeConfig::Combined { sources, .. }) => Some(sources.clone()),
            _ => None,
        }
    }
}

#[instrument(level = "debug")]
pub fn get_snippet_type(config: &Settings, name: &str) -> Result<SnippetType> {
    config.get_snippet_type(name).ok_or_else(|| {
        anyhow::anyhow!("Unknown snippet type: {}, update your configuration.", name)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn given_concrete_type_when_getting_snippet_type_then_returns_correct_type() {
        let mut snippet_types = HashMap::new();
        snippet_types.insert(
            "test".to_string(),
            SnippetTypeConfig::Concrete {
                source_file: PathBuf::from("test.txt"),
                description: None,
                alias: None,
                format: "default".to_string(),
            },
        );

        let settings = Settings {
            snippet_types,
            config_paths: vec![],
            active_config_path: None,
        };

        let snippet_type = settings.get_snippet_type("test");
        assert!(snippet_type.is_some());
        let snippet_type = snippet_type.unwrap();
        assert_eq!(snippet_type.name, "test");
        assert_eq!(snippet_type.source_file, PathBuf::from("test.txt"));
        assert_eq!(snippet_type.format, SnippetFormat::Default);
    }

    #[test]
    fn given_combined_type_when_getting_snippet_type_then_returns_none() {
        let mut snippet_types = HashMap::new();
        snippet_types.insert(
            "combined".to_string(),
            SnippetTypeConfig::Combined {
                sources: vec!["source1".to_string(), "source2".to_string()],
                description: None,
                alias: None,
            },
        );

        let settings = Settings {
            snippet_types,
            config_paths: vec![],
            active_config_path: None,
        };

        assert!(settings.get_snippet_type("combined").is_none());
    }

    #[test]
    fn given_combined_type_when_getting_sources_then_returns_source_list() {
        let mut snippet_types = HashMap::new();
        let sources = vec!["source1".to_string(), "source2".to_string()];
        snippet_types.insert(
            "combined".to_string(),
            SnippetTypeConfig::Combined {
                sources: sources.clone(),
                description: None,
                alias: None,
            },
        );

        let settings = Settings {
            snippet_types,
            config_paths: vec![],
            active_config_path: None,
        };

        let result = settings.get_combined_sources("combined");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), sources);
    }

    #[test]
    fn given_concrete_type_when_getting_sources_then_returns_none() {
        let mut snippet_types = HashMap::new();
        snippet_types.insert(
            "test".to_string(),
            SnippetTypeConfig::Concrete {
                source_file: PathBuf::from("test.txt"),
                description: None,
                alias: None,
                format: "default".to_string(),
            },
        );

        let settings = Settings {
            snippet_types,
            config_paths: vec![],
            active_config_path: None,
        };

        assert!(settings.get_combined_sources("test").is_none());
    }
}